
//
//
//
// Communication protocol to manage messages betwen MCU1 and MCU2
// - Shared Buffer
// - Data Structures
//
// Req:
//    - Structure for messages should include a message id, payload and a checksum
//    - MCU1 -> MCU2 uses a circular Buffer
//    - Functions to send and receive messages including calculating and verifying checksums
//
//    (Maybe) Helpful functions:
//    - calculate_checksum
//


use std::{collections::VecDeque, io::empty};



// Payload, message is and checksum
struct Message {
    id: u16,
    payload: Vec<u8>,
    checksum: u8,
}

impl Message {

    // create our new maessage and calculate checksum automatically
    fn new(id: u16, payload:Vec<u8>) -> Self {
        let checksum = Self::calculate_checksum(&payload);
        Message {
            id,
            payload,
            checksum,
        }
    }

    // XOR Checksum of payload bytes 
    // Ideally we'd do this byte-by-byte to minimize memory usage and processign overhead 
    fn calculate_checksum(payload: &[u8]) -> u8 {
        payload.iter().fold(0, |acc, byte| acc ^ byte)
    }

    // Simply verifies the messages integrity by recalculating the checksum 
    fn verify_checksum(&self) -> bool {
        self.checksum == Self::calculate_checksum(&self.payload)
    }
}

// Shared communication between MCU1->MCU2
 struct CircularBuffer {
    buffer: VecDeque<Message>,
    capacity: usize,
    write_count: usize,
    read_count: usize,
}

impl CircularBuffer {
    fn new(capacity: usize) -> Self {
        CircularBuffer {
            buffer: VecDeque::with_capacity(capacity),
            capacity, 
            write_count: 0, 
            read_count: 0,
        }
    }

    // Send message to buffer 
    // Ideally, we could do a few more things like block until space is available, return an error
    // on a full buffer or implement priority-based replacement
    fn send_message(&mut self, message: Message) -> Result<(), &'static str>  {
        // If the buffer is full, we should remove the oldest message (FIFO)
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }

        self.buffer.push_back(message);
        self.write_count += 1;
        Ok(())
    }

    // Receive message
    fn receive_message(&mut self) -> Option<Message> {
        if let Some(message) = self.buffer.pop_front() {
            self.read_count += 1;
            Some(message)
        } else {
            None
        }
    }

    // empty chec 
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    // full check 
    fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    // length of buffer 
    fn length(&self) -> usize {
        self.buffer.len()
    }
}


struct CommunicationProtocol {
    shared_buffer: CircularBuffer,
    next_message: u16,
}

impl CommunicationProtocol {
    fn new(buffer_capacity: usize) -> Self {
        CommunicationProtocol { 
            shared_buffer: CircularBuffer::new(buffer_capacity),
            next_message: 1, 
        }
    }

    fn mcu1_send(&mut self, payload: Vec<u8>) -> Result<u16, &'static str> {
        let message = Message::new(self.next_message, payload);
        let message_id = self.next_message;

        self.shared_buffer.send_message(message)?;
        self.next_message = self.next_message.wrapping_add(1);

        println!("MCU1 message sent- ID {}", message_id);
        Ok(message_id)
    }

    fn mcu2_receive(&mut self) -> Option<(Message, bool)> {
        if let Some(message) = self.shared_buffer.receive_message() {
            let valid_checksum = message.verify_checksum();
            if valid_checksum {
                println!("MCU2 message received with valid ID {}", message.id)
            } else {
                println!("MCU2 corrupted ID found {}", message.id)
            }

            Some((message, valid_checksum))
        } else {
            println!("MCU2: No messages available");
            None
        }
    }

    fn get_buffer_status(&self) -> (usize, bool, bool) {
        (self.shared_buffer.length(),
        self.shared_buffer.is_empty(),
        self.shared_buffer.is_full())
    }
}


fn main() {
    let mut comm_protocol = CommunicationProtocol::new(5);

    println!("====IPC Comms Test ====\n");

    let _ = comm_protocol.mcu1_send(vec![0x01, 0x02, 0x03]);
    let _ = comm_protocol.mcu1_send(vec![0x04, 0x05]);
    let _ = comm_protocol.mcu1_send(vec![0x06]);

    let (len, empty, full) = comm_protocol.get_buffer_status();
    println!("Buffer status: {} messages, empty: {}, full: {}\n", len, empty, full);

    while let Some((message, checksum_ok)) = comm_protocol.mcu2_receive() {
        if checksum_ok {
            println!("  Payload: {:?}", message.payload);
        } else {
            println!("  Checksum mismatch - data corruption detected.");
        }
    }

    println!("\n=== Test Complete ===");


    // println!("Hello, world!");
}
