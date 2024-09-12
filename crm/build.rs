use anyhow::Result;
use proto_builder_trait::tonic::BuilderAttributes;
use std::fs;

fn main() -> Result<()> {
    // 创建目标文件夹
    fs::create_dir_all("src/pb")?;
    // 配置Tonic
    tonic_build::configure()
        // 输出到src/pb
        .out_dir("src/pb")
        .with_derive_builder(&["WelcomeRequest", "RecallRequest", "RemindRequest"], None)
        .with_field_attributes(
            &["WelcomeRequest.content_ids"],
            &[r#"#[builder(setter(each(name="content_id",into)))]"#],
        )
        .compile(
            &["../protos/crm/messages.proto", "../protos/crm/rpc.proto"],
            &["../protos"],
        )?;

    Ok(())
}
