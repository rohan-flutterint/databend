// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(internal_features)]
#![feature(core_intrinsics)]
#![feature(impl_trait_in_assoc_type)]
#![feature(box_patterns)]
#![feature(iter_intersperse)]
#![allow(clippy::uninlined_format_args)]

mod append;
mod compression;
mod read;
mod stage_table;
mod streaming_load;
mod transform_generating;
mod transform_null_if;

pub use append::StageSinkTable;
pub use compression::get_compression_with_path;
pub use read::row_based::BytesBatch;
pub use stage_table::StageTable;
pub use streaming_load::build_streaming_load_pipeline;
pub use transform_null_if::TransformNullIf;
