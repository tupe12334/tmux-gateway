#![allow(
    unused_variables,
    dead_code,
    missing_docs,
    clippy::wildcard_imports,
    clippy::let_unit_value
)]
use tonic::codegen::*;

macro_rules! grpc_service {
    (
        package = $package:literal;
        service $trait_name:ident ($server_name:ident) {
            $(rpc $rpc_name:ident / $method:ident ( $req:ident ) -> $res:ident;)*
        }
    ) => {
        #[tonic::async_trait]
        pub trait $trait_name: Send + Sync + 'static {
            $(
                async fn $method(
                    &self,
                    request: tonic::Request<super::messages::$req>,
                ) -> Result<tonic::Response<super::messages::$res>, tonic::Status>;
            )*
        }

        #[derive(Debug)]
        pub struct $server_name<T> {
            inner: Arc<T>,
        }

        impl<T> $server_name<T> {
            pub fn new(inner: T) -> Self {
                Self {
                    inner: Arc::new(inner),
                }
            }
        }

        impl<T> Clone for $server_name<T> {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                }
            }
        }

        impl<T: $trait_name> tonic::server::NamedService for $server_name<T> {
            const NAME: &'static str = concat!($package, ".", stringify!($trait_name));
        }

        impl<T, B> Service<http::Request<B>> for $server_name<T>
        where
            T: $trait_name,
            B: Body + Send + 'static,
            B::Error: Into<StdError> + Send + 'static,
        {
            type Response = http::Response<tonic::body::Body>;
            type Error = std::convert::Infallible;
            type Future = BoxFuture<Self::Response, Self::Error>;

            fn poll_ready(
                &mut self,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: http::Request<B>) -> Self::Future {
                let inner = self.inner.clone();
                match req.uri().path() {
                    $(
                        concat!("/", $package, ".", stringify!($trait_name), "/", stringify!($rpc_name)) => {
                            struct Svc<T: $trait_name>(Arc<T>);
                            impl<T: $trait_name> tonic::server::UnaryService<super::messages::$req> for Svc<T> {
                                type Response = super::messages::$res;
                                type Future = BoxFuture<
                                    tonic::Response<Self::Response>,
                                    tonic::Status,
                                >;
                                fn call(
                                    &mut self,
                                    request: tonic::Request<super::messages::$req>,
                                ) -> Self::Future {
                                    let inner = Arc::clone(&self.0);
                                    Box::pin(async move {
                                        <T as $trait_name>::$method(&inner, request).await
                                    })
                                }
                            }
                            let fut = async move {
                                let method = Svc(inner);
                                let codec = tonic_prost::ProstCodec::default();
                                let mut grpc = tonic::server::Grpc::new(codec);
                                let res = grpc.unary(method, req).await;
                                Ok(res)
                            };
                            Box::pin(fut)
                        }
                    )*
                    _ => {
                        Box::pin(async move {
                            let mut response = http::Response::new(tonic::body::Body::default());
                            let headers = response.headers_mut();
                            headers.insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                            headers.insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                            Ok(response)
                        })
                    }
                }
            }
        }

        pub fn package_name() -> &'static str {
            $package
        }

        pub fn service_proto() -> &'static str {
            concat!(
                "service ", stringify!($trait_name), " {\n",
                $(
                    "  rpc ", stringify!($rpc_name),
                    "(", stringify!($req), ") returns (", stringify!($res), ");\n",
                )*
                "}\n",
            )
        }
    };
}

grpc_service! {
    package = "tmux_gateway";
    service TmuxGateway (TmuxGatewayServer) {
        rpc Ls / ls(LsRequest) -> LsResponse;
        rpc NewSession / new_session(NewSessionRequest) -> NewSessionResponse;
        rpc KillSession / kill_session(KillSessionRequest) -> KillSessionResponse;
        rpc KillWindow / kill_window(KillWindowRequest) -> KillWindowResponse;
        rpc KillPane / kill_pane(KillPaneRequest) -> KillPaneResponse;
        rpc ListWindows / list_windows(ListWindowsRequest) -> ListWindowsResponse;
        rpc ListPanes / list_panes(ListPanesRequest) -> ListPanesResponse;
        rpc SendKeys / send_keys(SendKeysRequest) -> SendKeysResponse;
        rpc RenameSession / rename_session(RenameSessionRequest) -> RenameSessionResponse;
        rpc RenameWindow / rename_window(RenameWindowRequest) -> RenameWindowResponse;
        rpc NewWindow / new_window(NewWindowRequest) -> NewWindowResponse;
        rpc SplitWindow / split_window(SplitWindowRequest) -> SplitWindowResponse;
        rpc CapturePane / capture_pane(CapturePaneRequest) -> CapturePaneResponse;
        rpc CapturePaneWithOptions / capture_pane_with_options(CapturePaneWithOptionsRequest) -> CapturePaneWithOptionsResponse;
        rpc CreateSessionWithWindows / create_session_with_windows(CreateSessionWithWindowsRequest) -> CreateSessionWithWindowsResponse;
        rpc SwapPanes / swap_panes(SwapPanesRequest) -> SwapPanesResponse;
        rpc MoveWindow / move_window(MoveWindowRequest) -> MoveWindowResponse;
        rpc SelectWindow / select_window(SelectWindowRequest) -> SelectWindowResponse;
        rpc SelectPane / select_pane(SelectPaneRequest) -> SelectPaneResponse;
        rpc ResizePane / resize_pane(ResizePaneRequest) -> ResizePaneResponse;
    }
}
