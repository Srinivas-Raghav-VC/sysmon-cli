extern crate sysinfo;
fn main(){
    println!("Minimum CPU update interval: {:?}", sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
}
