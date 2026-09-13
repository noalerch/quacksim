/// A model command with deterministic source metadata.
///
/// The engine orders envelopes by agent index and then by sequence before it
/// calls [`crate::TickModel::resolve_commands`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandEnvelope<C> {
    agent_index: usize,
    sequence: usize,
    command: C,
}

impl<C> CommandEnvelope<C> {
    /// Returns the index of the agent that created the command.
    #[must_use]
    pub const fn agent_index(&self) -> usize {
        self.agent_index
    }

    /// Returns the zero-based command sequence for the source agent.
    #[must_use]
    pub const fn sequence(&self) -> usize {
        self.sequence
    }

    /// Returns the model command.
    #[must_use]
    pub const fn command(&self) -> &C {
        &self.command
    }

    /// Returns the mutable model command.
    #[must_use]
    pub const fn command_mut(&mut self) -> &mut C {
        &mut self.command
    }

    /// Removes the envelope and returns the model command.
    #[must_use]
    pub fn into_command(self) -> C {
        self.command
    }
}

/// Collects commands from one agent behavior call.
///
/// A new sink starts with sequence zero. Each call to [`Self::push`] increases
/// the sequence by one.
pub struct CommandSink<'a, C> {
    agent_index: usize,
    next_sequence: usize,
    commands: &'a mut Vec<CommandEnvelope<C>>,
}

impl<'a, C> CommandSink<'a, C> {
    pub(crate) const fn new(agent_index: usize, commands: &'a mut Vec<CommandEnvelope<C>>) -> Self {
        Self {
            agent_index,
            next_sequence: 0,
            commands,
        }
    }

    /// Adds a command to the current agent command buffer.
    pub fn push(&mut self, command: C) {
        self.commands.push(CommandEnvelope {
            agent_index: self.agent_index,
            sequence: self.next_sequence,
            command,
        });
        self.next_sequence += 1;
    }

    /// Returns the number of commands added through this sink.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.next_sequence
    }

    /// Returns `true` when no command was added through this sink.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.next_sequence == 0
    }
}
