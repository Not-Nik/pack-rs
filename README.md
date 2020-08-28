# pack-rs
Read and write Minecraft packets everywhere

## Usage
Writing:
```rust
fn main()
{
    // Define my_stream here
    let mut packet = Packet::new(1);
    packet.write_var_int(42);

    my_stream.write_packet(&packet);
}
```
Reading:
```rust
fn main()
{
    // Define my_stream here
    let packet = my_stream.read_packet();
}
```
