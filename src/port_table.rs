use std::net::TcpStream;

fn is_port_free(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).is_err()
}

pub fn print_port_table(ports: &[(&str, u16, &str)]) {
    println!("┌──────────────┬───────┬────────┬─────────────────────────────────────────────┐");
    println!("│ Service      │ Port  │ Status │ Explorer                                    │");
    println!("├──────────────┼───────┼────────┼─────────────────────────────────────────────┤");
    for (name, port, explorer) in ports {
        let status = if is_port_free(*port) {
            "free"
        } else {
            "in use"
        };
        println!(
            "│ {:<12} │ {:<5} │ {:<6} │ {:<43} │",
            name, port, status, explorer
        );
    }
    println!("└──────────────┴───────┴────────┴─────────────────────────────────────────────┘");
}
