// Copyright 2022 Datafuse Labs.
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

use common_exception::Result;

use super::logic::LogicOperator;
use super::LogicFunction;
use crate::scalars::function_factory::FunctionFeatures;
use crate::scalars::Function2;
use crate::scalars::Function2Description;
#[derive(Clone)]
pub struct LogicNotFunction;

impl LogicNotFunction {
    pub fn try_create(_display_name: &str) -> Result<Box<dyn Function2>> {
        LogicFunction::try_create(LogicOperator::Not)
    }

    pub fn desc() -> Function2Description {
        let mut features = FunctionFeatures::default().num_arguments(1);
        features = features.deterministic();
        Function2Description::creator(Box::new(Self::try_create)).features(features)
    }
}