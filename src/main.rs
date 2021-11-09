mod postgres_driver;
use env_logger;
fn main() {
    env_logger::init();
    let driver = postgres_driver::driver::PostgresDriver{};
    driver.start();
}
