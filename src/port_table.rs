use std::net::TcpStream;

pub fn format_port_table(ports: &[(&str, u16, &str)]) -> String {
    // Determine the width of the Explorer column based on content (min 43).
    let explorer_width = ports
        .iter()
        .map(|(_, _, e)| e.len())
        .max()
        .unwrap_or(0)
        .max(43);

    let separator_top = format!(
        "┌──────────────┬───────┬────────┬─{}┐\n",
        "─".repeat(explorer_width + 2)
    );
    let header = format!(
        "│ Service      │ Port  │ Status │ {:<width$} │\n",
        "Explorer",
        width = explorer_width
    );
    let separator_mid = format!(
        "├──────────────┼───────┼────────┼─{}┤\n",
        "─".repeat(explorer_width + 2)
    );
    let separator_bot = format!(
        "└──────────────┴───────┴────────┴─{}┘\n",
        "─".repeat(explorer_width + 2)
    );

    let mut out = String::new();
    out.push_str(&separator_top);
    out.push_str(&header);
    out.push_str(&separator_mid);
    for (name, port, explorer) in ports {
        let status = if TcpStream::connect(("127.0.0.1", *port)).is_err() {
            "free"
        } else {
            "in use"
        };
        out.push_str(&format!(
            "│ {name:<12} │ {port:<5} │ {status:<6} │ {explorer:<explorer_width$} │\n"
        ));
    }
    out.push_str(&separator_bot);
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
