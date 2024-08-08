use fake::{faker::name::zh_cn::Name, Fake};

fn main() {
    let target = "丁志德";
    let mut count = 0_u64;
    let mut loop_count = 0_u64;
    loop {
        if count == u64::MAX {
            count = 0;
            loop_count += 1;
        }
        count += 1;
        let name: String = Name().fake();
        println!("{}", name);

        if name.contains(target) {
            println!("Found after {} - {} iterations", loop_count, count);
            return;
        }
    }
}
