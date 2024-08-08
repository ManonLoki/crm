use anyhow::Result;

fn main() -> Result<()> {
    std::fs::create_dir_all("src/pb")?;

    let builder = tonic_build::configure();

    builder.out_dir("src/pb").compile(
        &[
            "../protos/notification/messages.proto",
            "../protos/notification/rpc.proto",
        ],
        &["../protos"],
    )?;

    Ok(())
}
