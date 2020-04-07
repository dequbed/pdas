use clap::ArgMatches;

pub fn query(m: &ArgMatches) {
    let target = m.value_of("target").unwrap();
    println!("Would query {}", target);
}
