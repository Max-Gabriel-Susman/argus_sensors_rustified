use std::time::Duration;

use rclrs::*;
use ros_env::std_msgs::msg::String as StringMsg;

struct Talker {
    node: Node,
    _publisher: Publisher<StringMsg>,
    _timer: Timer,
}

impl Talker {
    fn new(executor: &executor) -> Result<Self, RclrsError> {
        let node = executor.create_node("talker")?;

        let publisher = node.create_publisher::<StringMsg>("chatter".keep_last(10))?;

        let timer_node = node.clone();
        let timer_publisher = publisher.clone();
        let mut count: u64 = 0;
        let timer = node.create_timer_repeating(Duration::from_millis(500), move || {
            count += 1;
            let msg = StringMsg {
                data: format!("Hello, world! {count}"),
            };
            log_info!(timer_node.logger(), "Publishing: '{}'", msg.data);
            if let Err(e) = timer_publisher.publish(msg) {
                log_error!(timer_node.logger(), "Failed to publish: {e}");
            }
        })?;

        Ok(Self {
            node,
            _publisher: publisher,
            _timer: timer,
        })
    }
}

fn main() -> Result<(), RclrsError> {
    println!("Initializing Argus Sensors...");

    let context = Context::default_from_env()?;
    let mut executor = context.create_basic_executor();

    let talker =

    println!("Argus Sensors online.");
}
