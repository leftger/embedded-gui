use heapless::Vec;

use crate::{GuiContext, GuiError, input::InputEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenId(pub u16);

impl ScreenId {
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenCommand {
    None,
    Push(ScreenId),
    Pop,
    Replace(ScreenId),
    ClearTo(ScreenId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenStackError {
    Full,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenLifecycleEvent {
    Mount(ScreenId),
    Unmount(ScreenId),
    Pause(ScreenId),
    Resume(ScreenId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenTransition {
    pub from: Option<ScreenId>,
    pub to: Option<ScreenId>,
    pub command: ScreenCommand,
}

pub trait Screen<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize> {
    fn id(&self) -> ScreenId;

    fn on_mount(
        &mut self,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        Ok(())
    }

    fn on_unmount(
        &mut self,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        Ok(())
    }

    fn on_pause(
        &mut self,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        Ok(())
    }

    fn on_resume(
        &mut self,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<(), GuiError> {
        Ok(())
    }

    fn handle_input(
        &mut self,
        _event: InputEvent,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<ScreenCommand, GuiError> {
        Ok(ScreenCommand::None)
    }

    fn tick(
        &mut self,
        _dt_ms: u32,
        _gui: &mut GuiContext<'a, NODES, EVENTS, DIRTY>,
    ) -> Result<ScreenCommand, GuiError> {
        Ok(ScreenCommand::None)
    }
}

pub struct ScreenStack<const N: usize> {
    stack: Vec<ScreenId, N>,
}

impl<const N: usize> ScreenStack<N> {
    pub const fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn with_root(root: ScreenId) -> Result<Self, ScreenStackError> {
        let mut stack = Self::new();
        stack.push(root)?;
        Ok(stack)
    }

    pub fn with_root_lifecycle<const M: usize>(
        root: ScreenId,
        events: &mut Vec<ScreenLifecycleEvent, M>,
    ) -> Result<Self, ScreenStackError> {
        let stack = Self::with_root(root)?;
        push_lifecycle(events, ScreenLifecycleEvent::Mount(root))?;
        Ok(stack)
    }

    pub fn push(&mut self, id: ScreenId) -> Result<(), ScreenStackError> {
        self.stack.push(id).map_err(|_| ScreenStackError::Full)
    }

    pub fn pop(&mut self) -> Result<ScreenId, ScreenStackError> {
        self.stack.pop().ok_or(ScreenStackError::Empty)
    }

    pub fn replace(&mut self, id: ScreenId) -> Result<(), ScreenStackError> {
        if self.stack.pop().is_none() {
            return Err(ScreenStackError::Empty);
        }
        self.push(id)
    }

    pub fn clear_to(&mut self, id: ScreenId) -> Result<(), ScreenStackError> {
        self.stack.clear();
        self.push(id)
    }

    pub fn apply(&mut self, command: ScreenCommand) -> Result<(), ScreenStackError> {
        match command {
            ScreenCommand::None => Ok(()),
            ScreenCommand::Push(id) => self.push(id),
            ScreenCommand::Pop => self.pop().map(|_| ()),
            ScreenCommand::Replace(id) => self.replace(id),
            ScreenCommand::ClearTo(id) => self.clear_to(id),
        }
    }

    pub fn apply_lifecycle<const M: usize>(
        &mut self,
        command: ScreenCommand,
        events: &mut Vec<ScreenLifecycleEvent, M>,
    ) -> Result<ScreenTransition, ScreenStackError> {
        let from = self.current();
        match command {
            ScreenCommand::None => Ok(ScreenTransition {
                from,
                to: from,
                command,
            }),
            ScreenCommand::Push(id) => {
                if let Some(current) = self.current() {
                    push_lifecycle(events, ScreenLifecycleEvent::Pause(current))?;
                }
                self.push(id)?;
                push_lifecycle(events, ScreenLifecycleEvent::Mount(id))?;
                Ok(ScreenTransition {
                    from,
                    to: self.current(),
                    command,
                })
            }
            ScreenCommand::Pop => {
                let old = self.pop()?;
                push_lifecycle(events, ScreenLifecycleEvent::Unmount(old))?;
                if let Some(current) = self.current() {
                    push_lifecycle(events, ScreenLifecycleEvent::Resume(current))?;
                }
                Ok(ScreenTransition {
                    from,
                    to: self.current(),
                    command,
                })
            }
            ScreenCommand::Replace(id) => {
                let old = self.pop()?;
                push_lifecycle(events, ScreenLifecycleEvent::Unmount(old))?;
                self.push(id)?;
                push_lifecycle(events, ScreenLifecycleEvent::Mount(id))?;
                Ok(ScreenTransition {
                    from,
                    to: self.current(),
                    command,
                })
            }
            ScreenCommand::ClearTo(id) => {
                while let Some(old) = self.stack.pop() {
                    push_lifecycle(events, ScreenLifecycleEvent::Unmount(old))?;
                }
                self.push(id)?;
                push_lifecycle(events, ScreenLifecycleEvent::Mount(id))?;
                Ok(ScreenTransition {
                    from,
                    to: self.current(),
                    command,
                })
            }
        }
    }

    pub fn current(&self) -> Option<ScreenId> {
        self.stack.last().copied()
    }

    pub fn as_slice(&self) -> &[ScreenId] {
        self.stack.as_slice()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

fn push_lifecycle<const M: usize>(
    events: &mut Vec<ScreenLifecycleEvent, M>,
    event: ScreenLifecycleEvent,
) -> Result<(), ScreenStackError> {
    events.push(event).map_err(|_| ScreenStackError::Full)
}

impl<const N: usize> Default for ScreenStack<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::style::Style;

    struct DummyScreen;

    impl<'a, const NODES: usize, const EVENTS: usize, const DIRTY: usize>
        Screen<'a, NODES, EVENTS, DIRTY> for DummyScreen
    {
        fn id(&self) -> ScreenId {
            ScreenId::new(1)
        }
    }

    #[test]
    fn screen_id_raw_and_default_lifecycle_hooks() {
        assert_eq!(ScreenId::new(7).raw(), 7);

        let mut gui = GuiContext::<4, 4, 4>::new(Rect::new(0, 0, 32, 32));
        gui.add_label(Rect::new(0, 0, 4, 4), "x", Style::label())
            .unwrap();
        let mut screen = DummyScreen;
        assert!(screen.on_mount(&mut gui).is_ok());
        assert!(screen.on_unmount(&mut gui).is_ok());
        assert!(screen.on_pause(&mut gui).is_ok());
        assert!(screen.on_resume(&mut gui).is_ok());
        assert_eq!(
            screen.handle_input(InputEvent::Select, &mut gui).unwrap(),
            ScreenCommand::None
        );
        assert_eq!(screen.tick(1, &mut gui).unwrap(), ScreenCommand::None);
    }

    #[test]
    fn screen_stack_default_clear_to_and_apply_none() {
        let mut stack = ScreenStack::<2>::default();
        assert!(stack.is_empty());
        stack.push(ScreenId::new(1)).unwrap();
        stack.push(ScreenId::new(2)).unwrap();
        stack.clear_to(ScreenId::new(3)).unwrap();
        assert_eq!(stack.as_slice(), &[ScreenId::new(3)]);
        assert_eq!(stack.current(), Some(ScreenId::new(3)));
        assert_eq!(stack.apply(ScreenCommand::None).unwrap(), ());
        assert!(stack.replace(ScreenId::new(4)).is_ok());
        assert!(stack.pop().is_ok());
        assert!(stack.replace(ScreenId::new(5)).is_err());
    }
}
