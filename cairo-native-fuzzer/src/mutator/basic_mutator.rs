use starknet_types_core::felt::Felt;

use crate::custom_rand::rng::Rng;
use crate::mutator::magic_values::MAGIC_VALUES;

/// This mutator only handles felt252
/// TODO: Handle more types
#[derive(Clone)]
pub struct Mutator {
    rng: Rng,
    max_input_size: usize,
}

impl Mutator {
    /// Creates a new `Mutator` with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Rng::seeded(seed),
            max_input_size: 252,
        }
    }

    pub fn mutate(&mut self, felt: Felt) -> Felt {
        // Perform a random mutation
        let mutation_type = self.rng.gen_range(0..=15); // Increase range to accommodate more strategies
        match mutation_type {
            0 => self.add_small_random_value(felt),
            1 => self.subtract_small_random_value(felt),
            2 => self.flip_random_bit(felt),
            3 => self.inc_byte(felt),
            4 => self.dec_byte(felt),
            // 5 => self.neg_byte(felt),
            6 => self.add_sub(felt),
            7 => self.swap(felt),
            8 => self.copy(felt),
            9 => self.inter_splice(felt),
            10 => self.magic_overwrite(felt),
            11 => self.magic_insert(felt),
            12 => self.random_overwrite(felt),
            13 => self.random_insert(felt),
            14 => self.byte_repeat_overwrite(felt),
            15 => self.byte_repeat_insert(felt),
            // Fallback to the original value if something goes wrong
            _ => felt,
        }
    }

    fn add_small_random_value(&mut self, felt: Felt) -> Felt {
        // Random value between 1 and 9
        let small_value = self.rng.gen_range(1..=9);
        felt + Felt::from(small_value)
    }

    fn subtract_small_random_value(&mut self, felt: Felt) -> Felt {
        // Random value between 1 and 9
        let small_value = self.rng.gen_range(1..=9);

        // Check for underflow before performing the subtraction
        if felt < Felt::from(small_value) {
            Felt::from(0)
        } else {
            felt - Felt::from(small_value)
        }
    }

    fn flip_random_bit(&mut self, felt: Felt) -> Felt {
        // Determine the actual bit length of the Felt value
        let felt_bytes = felt.to_bytes_be();
        let mut bit_length = 0;

        for byte in felt_bytes.iter().rev() {
            if *byte != 0 {
                bit_length =
                    (felt_bytes.len() - felt_bytes.iter().rev().position(|&b| b != 0).unwrap()) * 8;
                for i in (0..8).rev() {
                    if byte & (1 << i) != 0 {
                        bit_length += i + 1;
                        break;
                    }
                }
                break;
            }
        }

        if bit_length == 0 {
            // If the Felt value is zero, return the original value
            return felt;
        }

        // Random bit index within the actual bit length
        let bit_index = self.rng.gen_range(0..=bit_length - 1);
        let byte_index = bit_index / 8;
        let bit_position = bit_index % 8;

        // Ensure the byte index is within the valid range
        if byte_index >= felt_bytes.len() {
            return felt;
        }

        // Flip the bit at the calculated position
        let mut felt_bytes = felt.to_bytes_be();
        felt_bytes[byte_index] ^= 1 << bit_position;

        Felt::from_bytes_be(&felt_bytes)
    }

    fn inc_byte(&mut self, felt: Felt) -> Felt {
        felt + Felt::from(1)
    }

    fn dec_byte(&mut self, felt: Felt) -> Felt {
        // Check for underflow before performing the subtraction
        if felt <= Felt::from(0) {
            Felt::from(0)
        } else {
            felt - Felt::from(1)
        }
    }

    fn add_sub(&mut self, felt: Felt) -> Felt {
        // Add or subtract a random amount with a random endianness from a random size `u8` through `u64`
        let delta = self.rng.gen_range(0..=200) as i64 - 100; // Example range
        let new_felt = felt + Felt::from(delta);

        // Clamp the value to a reasonable range
        if new_felt > Felt::from(u64::MAX) {
            Felt::from(u64::MAX)
        } else if new_felt < Felt::from(0) {
            Felt::from(0)
        } else {
            new_felt
        }
    }

    fn swap(&mut self, felt: Felt) -> Felt {
        // Swap two ranges in an input buffer
        let mut felt_bytes = felt.to_bytes_be();
        let len = felt_bytes.len();
        let src = self.rng.gen_range(0..=len - 1);
        let dst = self.rng.gen_range(0..=len - 1);
        let swap_len = self.rng.gen_range(1..=len.min(len - src.max(dst)));

        for i in 0..swap_len {
            felt_bytes.swap(src + i, dst + i);
        }

        Felt::from_bytes_be(&felt_bytes)
    }

    fn copy(&mut self, felt: Felt) -> Felt {
        // Copy bytes from one location in the input and overwrite them at another
        let mut felt_bytes = felt.to_bytes_be();
        let len = felt_bytes.len();
        let src = self.rng.gen_range(0..=len - 1);
        let dst = self.rng.gen_range(0..=len - 1);
        let copy_len = self.rng.gen_range(1..=len.min(len - src.max(dst)));

        for i in 0..copy_len {
            felt_bytes[dst + i] = felt_bytes[src + i];
        }

        Felt::from_bytes_be(&felt_bytes)
    }

    fn inter_splice(&mut self, felt: Felt) -> Felt {
        // Take one location of the input and splice it into another
        let felt_bytes = felt.to_bytes_be();
        let len = felt_bytes.len();
        let src = self.rng.gen_range(0..=len - 1);
        let dst = self.rng.gen_range(0..=len - 1);
        let splice_len = self.rng.gen_range(1..=len.min(len - src.max(dst)));

        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&felt_bytes[..dst]);
        new_bytes.extend_from_slice(&felt_bytes[src..src + splice_len]);
        new_bytes.extend_from_slice(&felt_bytes[dst..]);

        // Ensure the length is exactly 32 bytes
        if new_bytes.len() > 32 {
            new_bytes.truncate(32);
        } else if new_bytes.len() < 32 {
            new_bytes.resize(32, 0);
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&new_bytes);

        Felt::from_bytes_be(&array)
    }

    fn magic_overwrite(&mut self, felt: Felt) -> Felt {
        // Pick a random magic value
        let magic_value = &MAGIC_VALUES[self.rng.gen_range(0..=MAGIC_VALUES.len() - 1)];
        let mut felt_bytes = felt.to_bytes_be();

        // Overwrite the bytes in the input with the magic value
        let len = magic_value.len().min(felt_bytes.len());
        felt_bytes[..len].copy_from_slice(&magic_value[..len]);

        Felt::from_bytes_be(&felt_bytes)
    }

    fn magic_insert(&mut self, felt: Felt) -> Felt {
        // Pick a random magic value
        let magic_value = &MAGIC_VALUES[self.rng.gen_range(0..=MAGIC_VALUES.len() - 1)];
        let felt_bytes = felt.to_bytes_be();

        // Insert the magic value at a random offset
        let offset = self.rng.gen_range(0..=felt_bytes.len());
        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&felt_bytes[..offset]);
        new_bytes.extend_from_slice(magic_value);
        new_bytes.extend_from_slice(&felt_bytes[offset..]);

        // Ensure the length is exactly 32 bytes
        if new_bytes.len() > 32 {
            new_bytes.truncate(32);
        } else if new_bytes.len() < 32 {
            new_bytes.resize(32, 0);
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&new_bytes);

        Felt::from_bytes_be(&array)
    }

    fn random_overwrite(&mut self, felt: Felt) -> Felt {
        // Overwrite a random offset of the input with random bytes
        let mut felt_bytes = felt.to_bytes_be();
        let offset = self.rng.gen_range(0..=felt_bytes.len() - 1);
        let amount = self.rng.gen_range(1..=felt_bytes.len() - offset);

        for i in offset..offset + amount {
            felt_bytes[i] = self.rng.rand_usize() as u8;
        }

        Felt::from_bytes_be(&felt_bytes)
    }

    fn random_insert(&mut self, felt: Felt) -> Felt {
        // Insert random bytes into a random offset in the input
        let felt_bytes = felt.to_bytes_be();
        let offset = self.rng.gen_range(0..=felt_bytes.len());
        let amount = self
            .rng
            .gen_range(0..=self.max_input_size - felt_bytes.len());

        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&felt_bytes[..offset]);
        new_bytes.extend(std::iter::repeat(self.rng.rand_usize() as u8).take(amount));
        new_bytes.extend_from_slice(&felt_bytes[offset..]);

        // Ensure the length is exactly 32 bytes
        if new_bytes.len() > 32 {
            new_bytes.truncate(32);
        } else if new_bytes.len() < 32 {
            new_bytes.resize(32, 0);
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&new_bytes);

        Felt::from_bytes_be(&array)
    }

    fn byte_repeat_overwrite(&mut self, felt: Felt) -> Felt {
        // Find a byte and repeat it multiple times by overwriting the data after it
        let mut felt_bytes = felt.to_bytes_be();
        let offset = self.rng.gen_range(0..=felt_bytes.len() - 1);
        let amount = self.rng.gen_range(1..=felt_bytes.len() - offset);

        let val = felt_bytes[offset];
        for i in offset + 1..offset + amount {
            felt_bytes[i] = val;
        }

        Felt::from_bytes_be(&felt_bytes)
    }

    fn byte_repeat_insert(&mut self, felt: Felt) -> Felt {
        // Find a byte and repeat it multiple times by splicing a random amount of the byte in
        let felt_bytes = felt.to_bytes_be();
        let offset = self.rng.gen_range(0..=felt_bytes.len() - 1);
        let amount = self
            .rng
            .gen_range(0..=self.max_input_size - felt_bytes.len());

        let val = felt_bytes[offset];
        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&felt_bytes[..offset]);
        new_bytes.extend(std::iter::repeat(val).take(amount));
        new_bytes.extend_from_slice(&felt_bytes[offset..]);

        // Ensure the length is exactly 32 bytes
        if new_bytes.len() > 32 {
            new_bytes.truncate(32);
        } else if new_bytes.len() < 32 {
            new_bytes.resize(32, 0);
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&new_bytes);

        Felt::from_bytes_be(&array)
    }
}
