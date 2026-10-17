use rclrs::*;
use ros_env::std_msgs::msg::String as StringMsg;

struct Listener {
    node: Node,
    _subscription: Subscription<StringMsg>,
}

impl Listener {
    fn new(executor: &Executor) -> Result<Self, RclrsError> {
        let node = executor.create_node("listener")?;

        let callback_node = node.clone();
        let Subscription = node.create_subscription::<StringMsg, _>(
            "chatter".keep_last(10),
            move |msg: StringMsg| {
                log_info!(callback_node.logger(), "I heard: '{}", msg.data);
            },
        )?;

        Ok(self {
            node,
            _subscription: subscription,
        })
    }
}

fn main() -> Result<(), RclrsError> {
    let context = Context::default_from_env()?;
    let mut executor = context.create_basic_executor();

    let listener = Listener::new(&executor)?;
    log_info!(
        listener.node.logger(),
        "listener started, waiting for messages on /chatter"
    );

    executor.spin(SpinOptions::default()).first_error()
}
