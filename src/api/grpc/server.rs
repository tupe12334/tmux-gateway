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
        $(#[doc = $service_doc:literal])*
        service $trait_name:ident ($server_name:ident) {
            $($items:tt)*
        }
    ) => {
        grpc_service!(@parse_items
            package = $package;
            service_docs = [$([$service_doc])*];
            trait_name = $trait_name;
            server_name = $server_name;
            unary_rpcs = [];
            stream_rpcs = [];
            $($items)*
        );
    };

    // Parse a documented unary rpc
    (@parse_items
        package = $package:literal;
        service_docs = [$($sd:tt)*];
        trait_name = $trait_name:ident;
        server_name = $server_name:ident;
        unary_rpcs = [$($unary:tt)*];
        stream_rpcs = [$($stream:tt)*];
        #[doc = $rpc_doc:literal]
        rpc $rpc_name:ident / $method:ident ( $req:ident ) -> $res:ident;
        $($rest:tt)*
    ) => {
        grpc_service!(@parse_items
            package = $package;
            service_docs = [$($sd)*];
            trait_name = $trait_name;
            server_name = $server_name;
            unary_rpcs = [$($unary)* { doc = [$rpc_doc]; name = $rpc_name; method = $method; req = $req; res = $res; }];
            stream_rpcs = [$($stream)*];
            $($rest)*
        );
    };

    // Parse an undocumented unary rpc
    (@parse_items
        package = $package:literal;
        service_docs = [$($sd:tt)*];
        trait_name = $trait_name:ident;
        server_name = $server_name:ident;
        unary_rpcs = [$($unary:tt)*];
        stream_rpcs = [$($stream:tt)*];
        rpc $rpc_name:ident / $method:ident ( $req:ident ) -> $res:ident;
        $($rest:tt)*
    ) => {
        grpc_service!(@parse_items
            package = $package;
            service_docs = [$($sd)*];
            trait_name = $trait_name;
            server_name = $server_name;
            unary_rpcs = [$($unary)* { doc = []; name = $rpc_name; method = $method; req = $req; res = $res; }];
            stream_rpcs = [$($stream)*];
            $($rest)*
        );
    };

    // Parse a documented stream rpc
    (@parse_items
        package = $package:literal;
        service_docs = [$($sd:tt)*];
        trait_name = $trait_name:ident;
        server_name = $server_name:ident;
        unary_rpcs = [$($unary:tt)*];
        stream_rpcs = [$($stream:tt)*];
        #[doc = $srpc_doc:literal]
        stream_rpc $srpc_name:ident / $smethod:ident [$stype:ident] ( $sreq:ident ) -> $sres:ident;
        $($rest:tt)*
    ) => {
        grpc_service!(@parse_items
            package = $package;
            service_docs = [$($sd)*];
            trait_name = $trait_name;
            server_name = $server_name;
            unary_rpcs = [$($unary)*];
            stream_rpcs = [$($stream)* { doc = [$srpc_doc]; name = $srpc_name; method = $smethod; stype = $stype; req = $sreq; res = $sres; }];
            $($rest)*
        );
    };

    // Parse an undocumented stream rpc
    (@parse_items
        package = $package:literal;
        service_docs = [$($sd:tt)*];
        trait_name = $trait_name:ident;
        server_name = $server_name:ident;
        unary_rpcs = [$($unary:tt)*];
        stream_rpcs = [$($stream:tt)*];
        stream_rpc $srpc_name:ident / $smethod:ident [$stype:ident] ( $sreq:ident ) -> $sres:ident;
        $($rest:tt)*
    ) => {
        grpc_service!(@parse_items
            package = $package;
            service_docs = [$($sd)*];
            trait_name = $trait_name;
            server_name = $server_name;
            unary_rpcs = [$($unary)*];
            stream_rpcs = [$($stream)* { doc = []; name = $srpc_name; method = $smethod; stype = $stype; req = $sreq; res = $sres; }];
            $($rest)*
        );
    };

    // Terminal: all items parsed, emit code
    (@parse_items
        package = $package:literal;
        service_docs = [$([$service_doc:literal])*];
        trait_name = $trait_name:ident;
        server_name = $server_name:ident;
        unary_rpcs = [$({ doc = [$($rpc_doc:literal)?]; name = $rpc_name:ident; method = $method:ident; req = $req:ident; res = $res:ident; })*];
        stream_rpcs = [$({ doc = [$($srpc_doc:literal)?]; name = $srpc_name:ident; method = $smethod:ident; stype = $stype:ident; req = $sreq:ident; res = $sres:ident; })*];
    ) => {
        #[tonic::async_trait]
        pub trait $trait_name: Send + Sync + 'static {
            $(
                async fn $method(
                    &self,
                    request: tonic::Request<super::messages::$req>,
                ) -> Result<tonic::Response<super::messages::$res>, tonic::Status>;
            )*
            $(
                type $stype: futures_core::Stream<Item = Result<super::messages::$sres, tonic::Status>> + Send + 'static;

                async fn $smethod(
                    &self,
                    request: tonic::Request<super::messages::$sreq>,
                ) -> Result<tonic::Response<Self::$stype>, tonic::Status>;
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

            #[allow(unknown_lints, no_wrapper_functions)]
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
                    $(
                        concat!("/", $package, ".", stringify!($trait_name), "/", stringify!($srpc_name)) => {
                            struct Svc<T: $trait_name>(Arc<T>);
                            impl<T: $trait_name> tonic::server::ServerStreamingService<super::messages::$sreq> for Svc<T> {
                                type Response = super::messages::$sres;
                                type ResponseStream = T::$stype;
                                type Future = BoxFuture<
                                    tonic::Response<Self::ResponseStream>,
                                    tonic::Status,
                                >;
                                fn call(
                                    &mut self,
                                    request: tonic::Request<super::messages::$sreq>,
                                ) -> Self::Future {
                                    let inner = Arc::clone(&self.0);
                                    Box::pin(async move {
                                        <T as $trait_name>::$smethod(&inner, request).await
                                    })
                                }
                            }
                            let fut = async move {
                                let method = Svc(inner);
                                let codec = tonic_prost::ProstCodec::default();
                                let mut grpc = tonic::server::Grpc::new(codec);
                                let res = grpc.server_streaming(method, req).await;
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

        pub fn service_proto() -> String {
            let mut s = String::new();
            $(
                s.push_str(concat!("// ", $service_doc, "\n"));
            )*
            s.push_str(concat!("service ", stringify!($trait_name), " {\n"));
            $(
                $(s.push_str(concat!("  // ", $rpc_doc, "\n"));)?
                s.push_str(concat!(
                    "  rpc ", stringify!($rpc_name),
                    "(", stringify!($req), ") returns (", stringify!($res), ");\n",
                ));
            )*
            $(
                $(s.push_str(concat!("  // ", $srpc_doc, "\n"));)?
                s.push_str(concat!(
                    "  rpc ", stringify!($srpc_name),
                    "(", stringify!($sreq), ") returns (stream ", stringify!($sres), ");\n",
                ));
            )*
            s.push_str("}\n");
            s
        }
    };
}

grpc_service! {
    package = "tmux_gateway";
    #[doc = "TmuxGateway provides a gRPC interface to tmux operations."]
    #[doc = "See: https://man.openbsd.org/tmux"]
    service TmuxGateway (TmuxGatewayServer) {
        #[doc = "List all sessions. See: https://man.openbsd.org/tmux#list-sessions"]
        rpc Ls / ls(LsRequest) -> LsResponse;
        #[doc = "Create a new session. See: https://man.openbsd.org/tmux#new-session"]
        rpc NewSession / new_session(NewSessionRequest) -> NewSessionResponse;
        #[doc = "Destroy a session. See: https://man.openbsd.org/tmux#kill-session"]
        rpc KillSession / kill_session(KillSessionRequest) -> KillSessionResponse;
        #[doc = "Destroy a window. See: https://man.openbsd.org/tmux#kill-window"]
        rpc KillWindow / kill_window(KillWindowRequest) -> KillWindowResponse;
        #[doc = "Destroy a pane. See: https://man.openbsd.org/tmux#kill-pane"]
        rpc KillPane / kill_pane(KillPaneRequest) -> KillPaneResponse;
        #[doc = "List windows in a session. See: https://man.openbsd.org/tmux#list-windows"]
        rpc ListWindows / list_windows(ListWindowsRequest) -> ListWindowsResponse;
        #[doc = "List panes in a window. See: https://man.openbsd.org/tmux#list-panes"]
        rpc ListPanes / list_panes(ListPanesRequest) -> ListPanesResponse;
        #[doc = "Send key(s) to a pane. See: https://man.openbsd.org/tmux#send-keys"]
        rpc SendKeys / send_keys(SendKeysRequest) -> SendKeysResponse;
        #[doc = "Rename a session. See: https://man.openbsd.org/tmux#rename-session"]
        rpc RenameSession / rename_session(RenameSessionRequest) -> RenameSessionResponse;
        #[doc = "Rename a window. See: https://man.openbsd.org/tmux#rename-window"]
        rpc RenameWindow / rename_window(RenameWindowRequest) -> RenameWindowResponse;
        #[doc = "Create a new window. See: https://man.openbsd.org/tmux#new-window"]
        rpc NewWindow / new_window(NewWindowRequest) -> NewWindowResponse;
        #[doc = "Split a pane to create a new pane. See: https://man.openbsd.org/tmux#split-window"]
        rpc SplitWindow / split_window(SplitWindowRequest) -> SplitWindowResponse;
        #[doc = "Capture pane contents. See: https://man.openbsd.org/tmux#capture-pane"]
        rpc CapturePane / capture_pane(CapturePaneRequest) -> CapturePaneResponse;
        #[doc = "Capture pane contents with options. See: https://man.openbsd.org/tmux#capture-pane"]
        rpc CapturePaneWithOptions / capture_pane_with_options(CapturePaneWithOptionsRequest) -> CapturePaneWithOptionsResponse;
        #[doc = "Create a session with multiple windows. See: https://man.openbsd.org/tmux#new-session"]
        rpc CreateSessionWithWindows / create_session_with_windows(CreateSessionWithWindowsRequest) -> CreateSessionWithWindowsResponse;
        #[doc = "Swap two panes. See: https://man.openbsd.org/tmux#swap-pane"]
        rpc SwapPanes / swap_panes(SwapPanesRequest) -> SwapPanesResponse;
        #[doc = "Move a window to another session. See: https://man.openbsd.org/tmux#move-window"]
        rpc MoveWindow / move_window(MoveWindowRequest) -> MoveWindowResponse;
        #[doc = "Select (activate) a window. See: https://man.openbsd.org/tmux#select-window"]
        rpc SelectWindow / select_window(SelectWindowRequest) -> SelectWindowResponse;
        #[doc = "Select (activate) a pane. See: https://man.openbsd.org/tmux#select-pane"]
        rpc SelectPane / select_pane(SelectPaneRequest) -> SelectPaneResponse;
        #[doc = "Resize a pane. See: https://man.openbsd.org/tmux#resize-pane"]
        rpc ResizePane / resize_pane(ResizePaneRequest) -> ResizePaneResponse;
        #[doc = "Apply a layout to a window. See: https://man.openbsd.org/tmux#select-layout"]
        rpc SelectLayout / select_layout(SelectLayoutRequest) -> SelectLayoutResponse;
        #[doc = "Get a tmux option. See: https://man.openbsd.org/tmux#show-options"]
        rpc GetOption / get_option(GetOptionRequest) -> GetOptionResponse;
        #[doc = "Set a tmux option. See: https://man.openbsd.org/tmux#set-option"]
        rpc SetOption / set_option(SetOptionRequest) -> SetOptionResponse;
        #[doc = "List tmux options. See: https://man.openbsd.org/tmux#show-options"]
        rpc ListOptions / list_options(ListOptionsRequest) -> ListOptionsResponse;
        #[doc = "Stream pane output. See: https://man.openbsd.org/tmux#capture-pane"]
        stream_rpc StreamPaneOutput / stream_pane_output [StreamPaneOutputStream] (StreamPaneOutputRequest) -> StreamPaneOutputResponse;
    }
}
