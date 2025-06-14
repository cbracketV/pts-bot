// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::Deserialize;

use evalexpr::error::EvalexprResult;
use evalexpr::eval_boolean_with_context;

use super::fn_context::FnContext;

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct Ets {
    #[serde(rename = "ETSVersion")]
    #[allow(dead_code)]
    pub version: Option<String>,
    #[serde(rename = "Profile")]
    pub profile: Profile,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "Group", default)]
    pub groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "Group", default)]
    pub groups: Vec<Group>,
    #[serde(rename = "TestCase", default)]
    pub testcases: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
pub struct TestCase {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Mapping", default)]
    pub mapping: String,
    #[serde(rename = "Description", default)]
    #[allow(dead_code)]
    pub description: String,
}

impl TestCase {
    pub fn is_enabled<F: Fn(&str) -> Option<bool>>(&self, get_value: &F) -> EvalexprResult<bool> {
        let mut mapping = self.mapping.replace("AND", "&&");
        mapping = mapping.replace("OR", "||");

        eval_boolean_with_context(&mapping, &FnContext(get_value))
    }
}

impl Group {
    pub fn testcases(&self) -> Box<dyn Iterator<Item = &TestCase> + '_> {
        Box::new(
            self.testcases
                .iter()
                .chain(self.groups.iter().flat_map(|group| group.testcases())),
        )
    }
}

impl XMLModel<'_> for Ets {
    const PATH: &'static str = "bin/Bluetooth/Ets/";
}

impl Ets {
    pub fn enabled_testcases<'a, F: 'a + Fn(&str) -> Option<bool>>(
        &'a self,
        get_value: F,
    ) -> impl Iterator<Item = String> + 'a {
        self.testcases()
            .filter(move |testcase| testcase.is_enabled(&get_value).unwrap_or(false))
            .map(|testcase| testcase.name.clone())
    }

    fn testcases(&self) -> impl Iterator<Item = &TestCase> + '_ {
        self.profile
            .groups
            .iter()
            .flat_map(|group| group.testcases())
    }
}
