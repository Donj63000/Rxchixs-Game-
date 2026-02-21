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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialEmoteIcon {
    TalkDots,
    Heart,
    Question,
    Laugh,
    Apology,
    Exclamation,
    Anger,
    Scribble,
    Lightning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialGesture {
    None,
    Talk,
    Wave,
    Explain,
    Laugh,
    Apologize,
    Threaten,
    Argue,
}

impl SocialActionKind {
    pub const ALL: [SocialActionKind; 10] = [
        SocialActionKind::DireBonjour,
        SocialActionKind::SmallTalk,
        SocialActionKind::Compliment,
        SocialActionKind::DemanderAide,
        SocialActionKind::Blague,
        SocialActionKind::Ragot,
        SocialActionKind::SExcuser,
        SocialActionKind::Menacer,
        SocialActionKind::Insulter,
        SocialActionKind::SEngueuler,
    ];

    pub const MENU_DEFAULT: [SocialActionKind; 10] = [
        SocialActionKind::DireBonjour,
        SocialActionKind::SmallTalk,
        SocialActionKind::Compliment,
        SocialActionKind::DemanderAide,
        SocialActionKind::Blague,
        SocialActionKind::Ragot,
        SocialActionKind::SExcuser,
        SocialActionKind::Insulter,
        SocialActionKind::SEngueuler,
        SocialActionKind::Menacer,
    ];

    pub fn ui_label(self) -> &'static str {
        match self {
            SocialActionKind::DireBonjour => "Dire bonjour",
            SocialActionKind::SmallTalk => "Discuter",
            SocialActionKind::Compliment => "Compliment",
            SocialActionKind::DemanderAide => "Demander aide",
            SocialActionKind::Blague => "Faire une blague",
            SocialActionKind::Ragot => "Ragot",
            SocialActionKind::SExcuser => "S'excuser",
            SocialActionKind::Menacer => "Menacer",
            SocialActionKind::Insulter => "Insulter",
            SocialActionKind::SEngueuler => "S'engueuler",
        }
    }

    pub fn is_positive(self) -> bool {
        matches!(
            self,
            SocialActionKind::DireBonjour
                | SocialActionKind::SmallTalk
                | SocialActionKind::Compliment
                | SocialActionKind::DemanderAide
                | SocialActionKind::Blague
                | SocialActionKind::SExcuser
        )
    }

    pub fn is_hostile(self) -> bool {
        matches!(
            self,
            SocialActionKind::Menacer | SocialActionKind::Insulter | SocialActionKind::SEngueuler
        )
    }

    pub fn duration_s(self) -> f32 {
        match self {
            SocialActionKind::DireBonjour => 0.9,
            SocialActionKind::SmallTalk => 2.8,
            SocialActionKind::Compliment => 2.0,
            SocialActionKind::DemanderAide => 2.2,
            SocialActionKind::Blague => 2.4,
            SocialActionKind::Ragot => 2.6,
            SocialActionKind::SExcuser => 2.2,
            SocialActionKind::Menacer => 1.6,
            SocialActionKind::Insulter => 1.7,
            SocialActionKind::SEngueuler => 3.2,
        }
    }

    pub fn emote_icon(self) -> SocialEmoteIcon {
        match self {
            SocialActionKind::DireBonjour => SocialEmoteIcon::TalkDots,
            SocialActionKind::SmallTalk => SocialEmoteIcon::TalkDots,
            SocialActionKind::Compliment => SocialEmoteIcon::Heart,
            SocialActionKind::DemanderAide => SocialEmoteIcon::Question,
            SocialActionKind::Blague => SocialEmoteIcon::Laugh,
            SocialActionKind::Ragot => SocialEmoteIcon::Scribble,
            SocialActionKind::SExcuser => SocialEmoteIcon::Apology,
            SocialActionKind::Menacer => SocialEmoteIcon::Exclamation,
            SocialActionKind::Insulter => SocialEmoteIcon::Anger,
            SocialActionKind::SEngueuler => SocialEmoteIcon::Lightning,
        }
    }

    pub fn gesture(self) -> SocialGesture {
        match self {
            SocialActionKind::DireBonjour => SocialGesture::Wave,
            SocialActionKind::SmallTalk => SocialGesture::Talk,
            SocialActionKind::Compliment => SocialGesture::Talk,
            SocialActionKind::DemanderAide => SocialGesture::Explain,
            SocialActionKind::Blague => SocialGesture::Laugh,
            SocialActionKind::Ragot => SocialGesture::Talk,
            SocialActionKind::SExcuser => SocialGesture::Apologize,
            SocialActionKind::Menacer => SocialGesture::Threaten,
            SocialActionKind::Insulter => SocialGesture::Argue,
            SocialActionKind::SEngueuler => SocialGesture::Argue,
        }
    }
}
