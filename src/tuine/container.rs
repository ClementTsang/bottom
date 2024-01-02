use crate::canvas::LayoutConstraint;

use super::Element;

/// A [`ContainerDirection`] determines the direction of the [`Container`].
pub enum ContainerDirection {
    Row,
    Column,
}

/// A [`Container`] holds either more containers or a [`BottomWidget`].
///
/// Basically, a non-leaf node in the [`Element`] tree.
pub struct Container {
    direction: ContainerDirection,
    constraint: LayoutConstraint,
    pub(super) children: Vec<Element>,
}

impl Container {
    pub fn draw(&mut self) {
        match self.direction {
            ContainerDirection::Row => {}
            ContainerDirection::Column => {}
        }
    }
}
