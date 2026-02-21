#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialActionKind {
    DireBonjour,
    SmallTalk,
    Compliment,
    DemanderAide,
    Blague,
    Ragot,
    SExcuser,
    Menacer,
    Insulter,
    SEngueuler,
}

impl SocialActionKind {
    pub const MENU_DEFAULT: [SocialActionKind; 10] = [
        Self::DireBonjour,
        Self::SmallTalk,
        Self::Compliment,
        Self::DemanderAide,
        Self::Blague,
        Self::Ragot,
        Self::SExcuser,
        Self::Insulter,
        Self::SEngueuler,
        Self::Menacer,
    ];

    pub fn ui_label(&self) -> &'static str {
        match self {
            Self::DireBonjour => "Dire bonjour",
            Self::SmallTalk => "Discuter",
            Self::Compliment => "Faire un compliment",
            Self::DemanderAide => "Demander un coup de main",
            Self::Blague => "Lacher une blague",
            Self::Ragot => "Balancer un ragot",
            Self::SExcuser => "S'excuser",
            Self::Menacer => "Menacer",
            Self::Insulter => "Insulter",
            Self::SEngueuler => "S'engueuler",
        }
    }

    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            Self::DireBonjour
                | Self::SmallTalk
                | Self::Compliment
                | Self::DemanderAide
                | Self::Blague
                | Self::SExcuser
        )
    }

    pub fn is_hostile(&self) -> bool {
        matches!(self, Self::Menacer | Self::Insulter | Self::SEngueuler)
    }
}
