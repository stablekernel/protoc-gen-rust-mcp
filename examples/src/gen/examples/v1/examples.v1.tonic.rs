// @generated
/// Generated client implementations.
pub mod vibe_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /** This is a complex comment to test string processing.
 It includes multiple lines with various characters:
 * Special chars: "quotes", 'single-quotes', \backslashes\
 * Symbols: @#$%^&*()_+-={}[]|;:<>,.?/
 * Newlines and     multiple    spaces

 It also has empty lines and indentation:
   - Indented item 1
   - Indented item 2
*/
    #[derive(Debug, Clone)]
    pub struct VibeServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl VibeServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> VibeServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> VibeServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            VibeServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /**
 This is a block comment
 with multiple lines
 to test block handling
 "Hello World", a `backtick`, and a path like C:\vibes\new
*/
        pub async fn set_vibe(
            &mut self,
            request: impl tonic::IntoRequest<super::SetVibeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/examples.v1.VibeService/SetVibe",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("examples.v1.VibeService", "SetVibe"));
            self.inner.unary(req, path, codec).await
        }
        /** Get Vibe
 of the server

*/
        pub async fn get_vibe(
            &mut self,
            request: impl tonic::IntoRequest<super::GetVibeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetVibeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/examples.v1.VibeService/GetVibe",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("examples.v1.VibeService", "GetVibe"));
            self.inner.unary(req, path, codec).await
        }
        /** Set vibe details
*/
        pub async fn set_vibe_details(
            &mut self,
            request: impl tonic::IntoRequest<super::SetVibeDetailsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/examples.v1.VibeService/SetVibeDetails",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("examples.v1.VibeService", "SetVibeDetails"));
            self.inner.unary(req, path, codec).await
        }
        /** Set the vibe arrays
*/
        pub async fn set_vibe_array(
            &mut self,
            request: impl tonic::IntoRequest<super::SetVibeArrayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeArrayResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/examples.v1.VibeService/SetVibeArray",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("examples.v1.VibeService", "SetVibeArray"));
            self.inner.unary(req, path, codec).await
        }
        /** Set multiple vibe objects
*/
        pub async fn set_vibe_objects(
            &mut self,
            request: impl tonic::IntoRequest<super::SetVibeObjectsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeObjectsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/examples.v1.VibeService/SetVibeObjects",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("examples.v1.VibeService", "SetVibeObjects"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod vibe_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with VibeServiceServer.
    #[async_trait]
    pub trait VibeService: std::marker::Send + std::marker::Sync + 'static {
        /**
 This is a block comment
 with multiple lines
 to test block handling
 "Hello World", a `backtick`, and a path like C:\vibes\new
*/
        async fn set_vibe(
            &self,
            request: tonic::Request<super::SetVibeRequest>,
        ) -> std::result::Result<tonic::Response<super::SetVibeResponse>, tonic::Status>;
        /** Get Vibe
 of the server

*/
        async fn get_vibe(
            &self,
            request: tonic::Request<super::GetVibeRequest>,
        ) -> std::result::Result<tonic::Response<super::GetVibeResponse>, tonic::Status>;
        /** Set vibe details
*/
        async fn set_vibe_details(
            &self,
            request: tonic::Request<super::SetVibeDetailsRequest>,
        ) -> std::result::Result<tonic::Response<super::SetVibeResponse>, tonic::Status>;
        /** Set the vibe arrays
*/
        async fn set_vibe_array(
            &self,
            request: tonic::Request<super::SetVibeArrayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeArrayResponse>,
            tonic::Status,
        >;
        /** Set multiple vibe objects
*/
        async fn set_vibe_objects(
            &self,
            request: tonic::Request<super::SetVibeObjectsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SetVibeObjectsResponse>,
            tonic::Status,
        >;
    }
    /** This is a complex comment to test string processing.
 It includes multiple lines with various characters:
 * Special chars: "quotes", 'single-quotes', \backslashes\
 * Symbols: @#$%^&*()_+-={}[]|;:<>,.?/
 * Newlines and     multiple    spaces

 It also has empty lines and indentation:
   - Indented item 1
   - Indented item 2
*/
    #[derive(Debug)]
    pub struct VibeServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> VibeServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for VibeServiceServer<T>
    where
        T: VibeService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/examples.v1.VibeService/SetVibe" => {
                    #[allow(non_camel_case_types)]
                    struct SetVibeSvc<T: VibeService>(pub Arc<T>);
                    impl<
                        T: VibeService,
                    > tonic::server::UnaryService<super::SetVibeRequest>
                    for SetVibeSvc<T> {
                        type Response = super::SetVibeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetVibeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VibeService>::set_vibe(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SetVibeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/examples.v1.VibeService/GetVibe" => {
                    #[allow(non_camel_case_types)]
                    struct GetVibeSvc<T: VibeService>(pub Arc<T>);
                    impl<
                        T: VibeService,
                    > tonic::server::UnaryService<super::GetVibeRequest>
                    for GetVibeSvc<T> {
                        type Response = super::GetVibeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetVibeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VibeService>::get_vibe(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetVibeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/examples.v1.VibeService/SetVibeDetails" => {
                    #[allow(non_camel_case_types)]
                    struct SetVibeDetailsSvc<T: VibeService>(pub Arc<T>);
                    impl<
                        T: VibeService,
                    > tonic::server::UnaryService<super::SetVibeDetailsRequest>
                    for SetVibeDetailsSvc<T> {
                        type Response = super::SetVibeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetVibeDetailsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VibeService>::set_vibe_details(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SetVibeDetailsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/examples.v1.VibeService/SetVibeArray" => {
                    #[allow(non_camel_case_types)]
                    struct SetVibeArraySvc<T: VibeService>(pub Arc<T>);
                    impl<
                        T: VibeService,
                    > tonic::server::UnaryService<super::SetVibeArrayRequest>
                    for SetVibeArraySvc<T> {
                        type Response = super::SetVibeArrayResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetVibeArrayRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VibeService>::set_vibe_array(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SetVibeArraySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/examples.v1.VibeService/SetVibeObjects" => {
                    #[allow(non_camel_case_types)]
                    struct SetVibeObjectsSvc<T: VibeService>(pub Arc<T>);
                    impl<
                        T: VibeService,
                    > tonic::server::UnaryService<super::SetVibeObjectsRequest>
                    for SetVibeObjectsSvc<T> {
                        type Response = super::SetVibeObjectsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SetVibeObjectsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as VibeService>::set_vibe_objects(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = SetVibeObjectsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for VibeServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "examples.v1.VibeService";
    impl<T> tonic::server::NamedService for VibeServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
