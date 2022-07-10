// Copyright 2022 Balaji (rbalajis25@gmail.com)
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
use tonic::{codegen::InterceptedService,service::Interceptor, transport::Channel};
pub mod apiproto {
    tonic::include_proto!("api"); // The string specified here must match the proto package name
}

#[derive(Clone)]
pub struct AuthInterceptor {
    pub token: String,
}

impl Interceptor for AuthInterceptor{
    fn call(&mut self,mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert("auth-token", self.token.clone().parse().unwrap());
        Ok(request)
    }
}

pub type InspektorClientCommon = InterceptedService<Channel, AuthInterceptor>;