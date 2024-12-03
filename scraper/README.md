# Running
1. Install cargo
2. `cargo run --release --bin articles`
    
    Run this until all articles are loaded.
    Requests will likely fail, so you will have to restart it.
3. `cargo run --release --bin links`
    
    Run this until all links are loaded.
    Requests will likely fail, so you will have to restart it.
4. Run the condensation SQL script `condensation.sql` to calculate
    the `condensed_links` column and remove links not in the graph.
    