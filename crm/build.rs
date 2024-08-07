use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    // 创建目标文件夹
    fs::create_dir_all("src/pb")?;
    // 配置Tonic
    tonic_build::configure()
        // 输出到src/pb
        .out_dir("src/pb")
        // 编译列表 crm.protocol
        // 包含其他数据 ../protos
        .compile(&["../protos/crm/crm.proto"], &["../protos"])?;

    Ok(())
}
