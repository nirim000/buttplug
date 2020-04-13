use super::TestDeviceImplCreator;
use crate::{
    core::errors::ButtplugError,
    server::device_manager::{
        DeviceCommunicationEvent, DeviceCommunicationManager, DeviceCommunicationManagerCreator,
    },
};
use async_std::{
    sync::{Sender, Arc, Mutex}
};
use async_trait::async_trait;

pub struct TestDeviceCommunicationManager {
    device_sender: Sender<DeviceCommunicationEvent>,
    devices: Arc<Mutex<Vec<Box<TestDeviceImplCreator>>>>
}

impl TestDeviceCommunicationManager {
    pub fn get_devices_clone(&self) -> Arc<Mutex<Vec<Box<TestDeviceImplCreator>>>> {
        self.devices.clone()
    }
}

impl DeviceCommunicationManagerCreator for TestDeviceCommunicationManager {
    fn new(device_sender: Sender<DeviceCommunicationEvent>) -> Self {
        Self { 
            device_sender,
            devices: Arc::new(Mutex::new(vec!()))
        }
    }
}

#[async_trait]
impl DeviceCommunicationManager for TestDeviceCommunicationManager {
    async fn start_scanning(&mut self) -> Result<(), ButtplugError> {
        let mut devices = self.devices.lock().await;
        if devices.is_empty() {
            panic!("No devices for test device comm manager to emit!");
        }
        while let Some(d) = devices.pop() {
            self.device_sender
                .send(DeviceCommunicationEvent::DeviceFound(d))
                .await;
        }
        Ok(())
    }

    async fn stop_scanning(&mut self) -> Result<(), ButtplugError> {
        Ok(())
    }

    fn is_scanning(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    mod test {
        use crate::{
            core::messages::{self, ButtplugOutMessage, ButtplugMessageSpecVersion},
            device::device::DeviceImpl,
            server::ButtplugServer,
            test::{TestDevice},
        };
        use async_std::{prelude::StreamExt, task};

        #[test]
        fn test_test_device_comm_manager() {
            let _ = env_logger::builder().is_test(true).try_init();
            let (mut server, mut recv) = ButtplugServer::new("Test Server", 0);
            let (device, device_creator) =
                TestDevice::new_bluetoothle_test_device_impl_creator("Massage Demo");
            
            task::block_on(async {
                let devices = server.add_test_comm_manager();
                devices.lock().await.push(Box::new(device_creator));
                let msg = messages::RequestServerInfo::new("Test Client", ButtplugMessageSpecVersion::Version2);
                let mut reply = server.parse_message(&msg.into()).await;
                assert!(reply.is_ok(), format!("Should get back ok: {:?}", reply));
                reply = server
                    .parse_message(&messages::StartScanning::default().into())
                    .await;
                assert!(reply.is_ok(), format!("Should get back ok: {:?}", reply));
                // Check that we got an event back about a new device.
                let msg = recv.next().await.unwrap();
                if let ButtplugOutMessage::DeviceAdded(da) = msg {
                    assert_eq!(da.device_name, "Aneros Vivi");
                } else {
                    assert!(
                        false,
                        format!(
                            "Returned message was not a DeviceAdded message or timed out: {:?}",
                            msg
                        )
                    );
                }
                device.disconnect().await;
                // Check that we got an event back about a removed device.
                let msg = recv.next().await.unwrap();
                if let ButtplugOutMessage::DeviceRemoved(da) = msg {
                    assert_eq!(da.device_index, 0);
                } else {
                    assert!(
                        false,
                        format!(
                            "Returned message was not a DeviceRemoved message or timed out: {:?}",
                            msg
                        )
                    );
                }
            });
        }
    }
}
