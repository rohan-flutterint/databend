//  Copyright 2021 Datafuse Labs.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use std::sync::Arc;

use common_exception::Result;
use common_expression::types::number::NumberScalar;
use common_expression::Chunk;
use common_expression::DataSchema;
use common_expression::DataSchemaRefExt;
use common_expression::Scalar;
use common_storages_table_meta::meta::Location;
use futures_util::TryStreamExt;

use crate::io::MetaReaders;
use crate::io::SegmentsIO;
use crate::io::SnapshotHistoryReader;
use crate::sessions::TableContext;
use crate::FuseTable;

pub struct FuseSegment<'a> {
    pub ctx: Arc<dyn TableContext>,
    pub table: &'a FuseTable,
    pub snapshot_id: String,
}

impl<'a> FuseSegment<'a> {
    pub fn new(ctx: Arc<dyn TableContext>, table: &'a FuseTable, snapshot_id: String) -> Self {
        Self {
            ctx,
            table,
            snapshot_id,
        }
    }

    pub async fn get_segments(&self) -> Result<Chunk> {
        let tbl = self.table;
        let maybe_snapshot = tbl.read_table_snapshot().await?;
        if let Some(snapshot) = maybe_snapshot {
            // prepare the stream of snapshot
            let snapshot_version = tbl.snapshot_format_version().await?;
            let snapshot_location = tbl
                .meta_location_generator
                .snapshot_location_from_uuid(&snapshot.snapshot_id, snapshot_version)?;
            let reader = MetaReaders::table_snapshot_reader(tbl.get_operator());
            let mut snapshot_stream = reader.snapshot_history(
                snapshot_location,
                snapshot_version,
                tbl.meta_location_generator().clone(),
            );

            // find the element by snapshot_id in stream
            while let Some(snapshot) = snapshot_stream.try_next().await? {
                if snapshot.snapshot_id.simple().to_string() == self.snapshot_id {
                    return self.to_block(&snapshot.segments).await;
                }
            }
        }

        Ok(Chunk::empty())
    }

    async fn to_block(&self, segment_locations: &[Location]) -> Result<Chunk> {
        let len = segment_locations.len();
        let mut format_versions: Vec<u64> = Vec::with_capacity(len);
        let mut block_count: Vec<u64> = Vec::with_capacity(len);
        let mut row_count: Vec<u64> = Vec::with_capacity(len);
        let mut compressed: Vec<u64> = Vec::with_capacity(len);
        let mut uncompressed: Vec<u64> = Vec::with_capacity(len);
        let mut file_location: Vec<Vec<u8>> = Vec::with_capacity(len);

        let segments_io = SegmentsIO::create(self.ctx.clone(), self.table.operator.clone());
        let segments = segments_io.read_segments(segment_locations).await?;
        for (idx, segment) in segments.iter().enumerate() {
            let segment = segment.clone()?;
            format_versions.push(segment_locations[idx].1);
            block_count.push(segment.summary.block_count);
            row_count.push(segment.summary.row_count);
            compressed.push(segment.summary.compressed_byte_size);
            uncompressed.push(segment.summary.uncompressed_byte_size);
            file_location.push(segment_locations[idx].0.clone().into_bytes());
        }

        Ok(Chunk::new(
            vec![
                (
                    Value::Scalar(Scalar::String(file_location.to_vec())),
                    DataType::String,
                ),
                (
                    Value::Scalar(Scalar::Number(NumberScalar::UInt64(format_versions))),
                    DataType::Number(NumberDataType::UInt64),
                ),
                (
                    Value::Scalar(Scalar::Number(NumberScalar::UInt64(block_count))),
                    DataType::Number(NumberDataType::UInt64),
                ),
                (
                    Value::Scalar(Scalar::Number(NumberScalar::UInt64(row_count))),
                    DataType::Number(NumberDataType::UInt64),
                ),
                (
                    Value::Scalar(Scalar::Number(NumberScalar::UInt64(uncompressed))),
                    DataType::Number(NumberDataType::UInt64),
                ),
                (
                    Value::Scalar(Scalar::Number(NumberScalar::UInt64(compressed))),
                    DataType::Number(NumberDataType::UInt64),
                ),
            ],
            1,
        ))
    }

    pub fn schema() -> Arc<DataSchema> {
        DataSchemaRefExt::create(vec![
            DataField::new("file_location", SchemaDataType::String),
            DataField::new(
                "format_version",
                SchemaDataType::Number(NumberDataType::UInt64),
            ),
            DataField::new(
                "block_count",
                SchemaDataType::Number(NumberDataType::UInt64),
            ),
            DataField::new("row_count", SchemaDataType::Number(NumberDataType::UInt64)),
            DataField::new(
                "bytes_uncompressed",
                SchemaDataType::Number(NumberDataType::UInt64),
            ),
            DataField::new(
                "bytes_compressed",
                SchemaDataType::Number(NumberDataType::UInt64),
            ),
        ])
    }
}
