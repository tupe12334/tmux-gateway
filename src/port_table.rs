use std::net::TcpStream;

fn is_port_free(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).is_err()
}

pub fn format_port_table(ports: &[(&str, u16, &str)]) -> String {
    let mut out = String::new();
    out.push_str(
        "┌──────────────┬───────┬────────┬─────────────────────────────────────────────┐\n",
    );
    out.push_str(
        "│ Service      │ Port  │ Status │ Explorer                                    │\n",
    );
    out.push_str(
        "├──────────────┼───────┼────────┼─────────────────────────────────────────────┤\n",
    );
    for (name, port, explorer) in ports {
        let status = if is_port_free(*port) {
            "free"
        } else {
            "in use"
        };
        out.push_str(&format!(
            "│ {:<12} │ {:<5} │ {:<6} │ {:<43} │\n",
            name, port, status, explorer
        ));
    }
    out.push_str(
        "└──────────────┴───────┴────────┴─────────────────────────────────────────────┘\n",
    );
    out
}

pub fn print_port_table(ports: &[(&str, u16, &str)]) {
    print!("{}", format_port_table(ports));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_contains_header_and_borders() {
        let table = format_port_table(&[("REST", 3000, "http://localhost:3000")]);
        assert!(table.contains("Service"));
        assert!(table.contains("Port"));
        assert!(table.contains("Status"));
        assert!(table.contains("Explorer"));
        assert!(table.starts_with('┌'));
        assert!(table.trim_end().ends_with('┘'));
    }

    #[test]
    fn table_contains_service_entry() {
        let table = format_port_table(&[("REST", 3000, "http://localhost:3000/swagger-ui")]);
        assert!(table.contains("REST"));
        assert!(table.contains("3000"));
        assert!(table.contains("http://localhost:3000/swagger-ui"));
    }

    #[test]
    fn table_with_multiple_entries() {
        let table = format_port_table(&[
            ("REST", 3000, "http://localhost:3000"),
            ("gRPC", 50051, "grpcui localhost:50051"),
        ]);
        assert!(table.contains("REST"));
        assert!(table.contains("gRPC"));
        assert!(table.contains("3000"));
        assert!(table.contains("50051"));
    }

    #[test]
    fn empty_table_has_header_only() {
        let table = format_port_table(&[]);
        assert!(table.contains("Service"));
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn status_shows_free_or_in_use() {
        let table = format_port_table(&[("Test", 1, "n/a")]);
        assert!(table.contains("free") || table.contains("in use"));
    }
}
