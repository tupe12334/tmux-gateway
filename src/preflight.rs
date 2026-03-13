use std::env;
use std::fmt;
use std::net::TcpListener;
use std::path::Path;
use tmux_gateway_core::tmux_interface::Tmux;

pub struct ServerConfig {
    pub http_port: u16,
    pub grpc_port: u16,
}

enum Status {
    Pass,
    Warn,
    Fail,
}

struct Check {
    name: String,
    status: Status,
    message: String,
}

impl fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon = match self.status {
            Status::Pass => "✓",
            Status::Warn => "⚠",
            Status::Fail => "✗",
        };
        write!(f, "  {icon} {:<22} {}", self.name, self.message)
    }
}

pub fn run() -> ServerConfig {
    let mut checks: Vec<Check> = Vec::new();
    let mut http_port: Option<u16> = None;
    let mut grpc_port: Option<u16> = None;

    // ── tmux binary ──────────────────────────────────────────────
    match Tmux::new().version().output() {
        Ok(output) => {
            let raw = output.into_inner();
            if raw.status.success() {
                let version = String::from_utf8_lossy(&raw.stdout).trim().to_string();
                checks.push(Check {
                    name: "tmux binary".into(),
                    status: Status::Pass,
                    message: version,
                });
            } else {
                let stderr = String::from_utf8_lossy(&raw.stderr).trim().to_string();
                checks.push(Check {
                    name: "tmux binary".into(),
                    status: Status::Fail,
                    message: format!("tmux returned error: {stderr}"),
                });
            }
        }
        Err(_) => {
            checks.push(Check {
                name: "tmux binary".into(),
                status: Status::Fail,
                message: "not found on PATH — install with: brew install tmux".into(),
            });
        }
    }

    // ── HTTP_PORT ────────────────────────────────────────────────
    check_port_env("HTTP_PORT", &mut checks, &mut http_port);

    // ── GRPC_PORT ────────────────────────────────────────────────
    check_port_env("GRPC_PORT", &mut checks, &mut grpc_port);

    // ── port conflict ────────────────────────────────────────────
    if let (Some(h), Some(g)) = (http_port, grpc_port)
        && h == g
    {
        checks.push(Check {
            name: "port conflict".into(),
            status: Status::Fail,
            message: format!("HTTP_PORT and GRPC_PORT are both {h} — they must differ"),
        });
    }

    // ── port availability ────────────────────────────────────────
    if let Some(port) = http_port {
        check_port_available("HTTP port available", port, &mut checks);
    }
    if let Some(port) = grpc_port {
        check_port_available("gRPC port available", port, &mut checks);
    }

    // ── schemas directory ────────────────────────────────────────
    check_schemas_dir(&mut checks);

    // ── print results ────────────────────────────────────────────
    let has_failure = checks.iter().any(|c| matches!(c.status, Status::Fail));

    println!();
    println!("  Preflight checks");
    println!("  ────────────────");
    for check in &checks {
        println!("{check}");
    }
    println!();

    if has_failure {
        tracing::error!("Preflight failed — fix the issues above and try again");
        std::process::exit(1);
    }

    ServerConfig {
        http_port: http_port.unwrap(),
        grpc_port: grpc_port.unwrap(),
    }
}

fn check_port_env(var: &str, checks: &mut Vec<Check>, out: &mut Option<u16>) {
    match env::var(var) {
        Ok(val) => match val.parse::<u16>() {
            Ok(0) => {
                checks.push(Check {
                    name: var.into(),
                    status: Status::Fail,
                    message: "port 0 is not valid".into(),
                });
            }
            Ok(port) => {
                let status = if port < 1024 {
                    Status::Warn
                } else {
                    Status::Pass
                };
                let message = if port < 1024 {
                    format!("{port} (privileged — may require elevated permissions)")
                } else {
                    port.to_string()
                };
                checks.push(Check {
                    name: var.into(),
                    status,
                    message,
                });
                *out = Some(port);
            }
            Err(_) => {
                checks.push(Check {
                    name: var.into(),
                    status: Status::Fail,
                    message: format!("\"{val}\" is not a valid port number"),
                });
            }
        },
        Err(_) => {
            checks.push(Check {
                name: var.into(),
                status: Status::Fail,
                message: format!("not set — add {var} to .env or environment"),
            });
        }
    }
}

fn check_port_available(name: &str, port: u16, checks: &mut Vec<Check>) {
    match TcpListener::bind(format!("0.0.0.0:{port}")) {
        Ok(_listener) => {
            // listener is dropped here, releasing the port
            checks.push(Check {
                name: name.into(),
                status: Status::Pass,
                message: format!(":{port} is free"),
            });
        }
        Err(e) => {
            checks.push(Check {
                name: name.into(),
                status: Status::Fail,
                message: format!(":{port} — {e}"),
            });
        }
    }
}

fn check_schemas_dir(checks: &mut Vec<Check>) {
    let schemas_dir = Path::new("schemas");
    match std::fs::create_dir_all(schemas_dir) {
        Ok(_) => {
            let test_file = schemas_dir.join(".preflight_check");
            match std::fs::write(&test_file, b"") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    checks.push(Check {
                        name: "schemas directory".into(),
                        status: Status::Pass,
                        message: "writable".into(),
                    });
                }
                Err(e) => {
                    checks.push(Check {
                        name: "schemas directory".into(),
                        status: Status::Fail,
                        message: format!("not writable — {e}"),
                    });
                }
            }
        }
        Err(e) => {
            checks.push(Check {
                name: "schemas directory".into(),
                status: Status::Fail,
                message: format!("cannot create — {e}"),
            });
        }
    }
}
