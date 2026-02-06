fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "../hcp-server/api/proto/transaction.proto",
                "../hcp-server/api/proto/block.proto",
                "../hcp-server/api/proto/common.proto",
            ],
            &["../hcp-server"],
        )?;
    Ok(())
}
