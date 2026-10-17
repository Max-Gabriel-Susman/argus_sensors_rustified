# Argus Sensors Rustified

Argus Sensors but... Rusty.

## Build & Launch

Build and launch the package nodes, your ROS workspace may be in a different location:
```
cd ~/Documents/argus_ws
colcon build --packages-select argus_sensors_rustified
source install/setup.bash
ros2 launch argus_sensors_rustified argus_sensors_rustified.launch.xml
```
