// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Winlock is an LLM agent coding assistant named after Anna Winlock.
//!
//! Anna Winlock was a pioneering woman in computing who worked at Harvard College
//! Observatory in the late 1800s. She was known for her mathematical and
//! computational work in astronomy.

/// The main agent type that coordinates interactions with the LLM.
#[derive(Debug)]
pub struct Agent {
    // Fields will be added as we implement functionality
}

impl Agent {
    /// Creates a new agent instance.
    pub fn new() -> Self {
        Self {}
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new();
        assert!(std::mem::size_of_val(&agent) > 0);
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
