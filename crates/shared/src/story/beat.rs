//! Beats: the small happenings a story is told in ("the relative went to the
//! place and never came back"). Each language has one or more wordings of
//! each (`beat-<key>`, its attributes), so a beat is a value, never words.
//! From the research catalogue (`docs/context/research-story-plots.md`): the
//! catalogue's codes are in each value's comment.

use super::fact::fact;
use super::facts::Reason;

fact! {
    /// One beat of a story.
    Beat as "beat" {
        // Hooks and backstory (paragraph 1).
        /// H1: the relative went to the place on an errand and never came back.
        RelativeWent = "relative-went",
        /// H2: something of the relative's was lost at the place.
        HeirloomLost = "heirloom-lost",
        /// H3: the family lived or worked there until creatures drove them off.
        DrivenOff = "driven-off",
        /// H4: the pet ran off towards the place.
        PetStrayed = "pet-strayed",
        /// H5: someone told of something at the place.
        RumourHeard = "rumour-heard",
        /// H6: the rival wronged the giver.
        NeighbourWrong = "neighbour-wrong",
        /// H7: a debt between the giver and someone in another town.
        DebtOwed = "debt-owed",
        /// H8: an order needs goods or materials.
        OrderPending = "order-pending",
        /// H9: creatures were seen near town.
        ThreatSeen = "threat-seen",
        /// H10: the relative died at the place and was never brought home.
        DiedThere = "died-there",
        /// H11: the relative asked something of the giver before dying.
        DyingWish = "dying-wish",
        /// H12: the giver went there long ago and fled.
        GiverWasThere = "giver-was-there",
        /// H13: what the place once was.
        PlaceHistory = "place-history",
        /// H14: a named creature leads them.
        BossNamed = "boss-named",
        /// H15: someone at home is ill and needs a remedy.
        SickAtHome = "sick-at-home",
        /// H16: the relative went off with someone the family disapproved of.
        Eloped = "eloped",
        /// Learning (owner): the giver wants something studied or practised.
        StudyWanted = "study-wanted",
        /// D1 (dark): the relative was killed by the rival.
        Murdered = "murdered",

        // Complications (paragraph 2).
        /// C1: someone went before and came back hurt, or didn't.
        FirstTryFailed = "first-try-failed",
        /// C2: since then, creatures have taken the place.
        CreaturesMovedIn = "creatures-moved-in",
        /// C3: the giver is too old, hurt or afraid to go.
        GiverCannot = "giver-cannot",
        /// C4: the occasion is soon, or the debt falls due.
        TimePressure = "time-pressure",
        /// C5: the giver asks that the rival or family not be told.
        KeepItQuiet = "keep-it-quiet",
        /// C6: the town laughs at the rumour.
        NobodyBelieves = "nobody-believes",
        /// C7: the rival is after the same thing.
        RivalSeeksIt = "rival-seeks-it",
        /// C8: the giver's family is blamed.
        Accused = "accused",
        /// C9: nobody knows whether the relative lives.
        FateUnknown = "fate-unknown",
        /// C10: the giver can't afford the price.
        CannotPay = "cannot-pay",
        /// C11: the giver caused it (left the gate open for the pet).
        Guilt = "guilt",
        /// C12: they searched the wrong place before.
        WrongPlaceSearched = "wrong-place-searched",
        /// C13: the weather makes the way hard (higher rewards).
        WeatherBars = "weather-bars",
        /// T1 (later steps): the rival wasn't the wrongdoer after all.
        NotGuilty = "not-guilty",
        /// T2 (later steps): the lost relative left by choice.
        LeftOnPurpose = "left-on-purpose",
        /// T3 (later steps): the named creature was once someone from town.
        BossWasOneOfUs = "boss-was-one-of-us",
        /// Q1 (later steps): two people want opposite things.
        WhoToHelp = "who-to-help",
        /// Q2 (later steps): keep the find, or return it to its first owner.
        KeepOrReturn = "keep-or-return",
        /// Q3 (later steps): the boss or the rival asks to be spared.
        SpareOrSlay = "spare-or-slay",

        // The ask (paragraph 3, always).
        /// A1: bring the item back.
        Retrieve = "retrieve",
        /// A2: kill the creatures or the boss.
        Clear = "clear",
        /// A3: go and see what became of them.
        LearnFate = "learn-fate",
        /// A4: take this to the recipient.
        Take = "take",
        /// A5: see the relative safely there or home.
        Bring = "bring",
        /// A5, for the pet: bring it home.
        BringPet = "bring-pet",
        /// A6: gather a number of herbs or materials.
        Collect = "collect",
        /// A7: look at the place and tell the giver what is there.
        Look = "look",
        /// A8: bring back a keepsake for burial.
        LayToRest = "lay-to-rest",
        /// D3 (dark): take something from the rival.
        StealFrom = "steal-from",
        /// D4 (dark): plant something to have the rival blamed.
        Frame = "frame",
        /// D5 (dark): make a witness keep quiet.
        Silence = "silence",

        // The stake (paragraph 3).
        /// S1: it is wanted for an occasion.
        ForOccasion = "for-occasion",
        /// S2: something to remember them by.
        Memory = "memory",
        /// S3: before they come to town.
        Safety = "safety",
        /// S4: to prove the rival wrong.
        Honour = "honour",
        /// S5: or the farm, shop or trade is lost.
        Livelihood = "livelihood",
        /// S6: a child misses the pet or the parent.
        ChildMisses = "child-misses",
        /// S7: a promise to the dead.
        Promise = "promise",
        /// S8: the real reward, named (owner).
        Reward = "reward",
        /// S10: before the season turns (higher rewards).
        SeasonTurns = "season-turns",
        /// S9: a warning about the creatures.
        Warning = "warning",
        /// D2 (dark): the giver wants the wrongdoer to pay.
        Revenge = "revenge",

        // The ending, told at turn-in (Propp's return).
        /// Thanks, always.
        Thanks = "thanks",
        /// What was found (after "go and see").
        Found = "found",
        /// The stake met ("the wedding can go ahead").
        StakeMet = "stake-met",
        /// L3: a lie comes out.
        TruthFound = "truth-found",
        /// A lead to the chain's next step.
        Lead = "lead",
    }
}

/// Where a beat goes: paragraph 1 (hook), 2 (complication), 3 (ask, stake),
/// or the ending told at turn-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    Hook,
    Complication,
    Ask,
    Stake,
    Ending,
}

impl Beat {
    pub fn slot(self) -> Slot {
        use Beat::*;
        match self {
            RelativeWent | HeirloomLost | DrivenOff | PetStrayed | RumourHeard | NeighbourWrong
            | DebtOwed | OrderPending | ThreatSeen | DiedThere | DyingWish | GiverWasThere
            | PlaceHistory | BossNamed | SickAtHome | Eloped | StudyWanted | Murdered => Slot::Hook,
            Retrieve | Clear | LearnFate | Take | Bring | BringPet | Collect | Look | LayToRest
            | StealFrom | Frame | Silence => Slot::Ask,
            ForOccasion | Memory | Safety | Honour | Livelihood | ChildMisses | Promise
            | Reward | SeasonTurns | Warning | Revenge => Slot::Stake,
            Thanks | Found | StakeMet | TruthFound | Lead => Slot::Ending,
            _ => Slot::Complication,
        }
    }

    /// Dark beats: only in quests needing a bad reputation (owner).
    pub fn is_dark(self) -> bool {
        use Beat::*;
        matches!(self, Murdered | Revenge | StealFrom | Frame | Silence)
    }

    /// Twists and quandaries: only in later steps of a chain (owner).
    pub fn is_twist(self) -> bool {
        use Beat::*;
        matches!(
            self,
            NotGuilty | LeftOnPurpose | BossWasOneOfUs | WhoToHelp | KeepOrReturn | SpareOrSlay
        )
    }

    /// The reasons a beat suits; empty means any.
    pub fn reasons(self) -> &'static [Reason] {
        use Beat::*;
        use Reason::*;
        match self {
            RelativeWent => &[Family, Grief],
            HeirloomLost => &[Family, Grief],
            DrivenOff => &[Family, Threat, DebtOrTrade],
            PetStrayed => &[LostPet],
            RumourHeard => &[Rumour, Threat],
            NeighbourWrong => &[NeighbourFeud],
            DebtOwed => &[DebtOrTrade],
            OrderPending => &[DebtOrTrade, Learning],
            ThreatSeen => &[Threat],
            DiedThere | DyingWish => &[Grief, Family],
            GiverWasThere => &[Rumour, Grief, Threat],
            BossNamed => &[Threat, Rumour, Grief],
            SickAtHome => &[Family, DebtOrTrade, Learning],
            Eloped => &[Family],
            StudyWanted => &[Learning],
            Murdered => &[Grief, NeighbourFeud],
            CreaturesMovedIn => &[Family, Grief, Rumour, DebtOrTrade],
            TimePressure => &[Family, DebtOrTrade, Threat],
            KeepItQuiet => &[NeighbourFeud, DebtOrTrade],
            NobodyBelieves => &[Rumour, Threat],
            RivalSeeksIt => &[Rumour, NeighbourFeud, Grief],
            Accused => &[NeighbourFeud],
            FateUnknown => &[Family, Grief],
            CannotPay => &[DebtOrTrade],
            Guilt => &[LostPet],
            WrongPlaceSearched => &[Family, LostPet, Rumour],
            ForOccasion => &[Family, DebtOrTrade],
            Memory => &[Grief, Family],
            Safety => &[Threat, LostPet],
            Honour => &[NeighbourFeud],
            Livelihood => &[DebtOrTrade, Threat],
            ChildMisses => &[LostPet, Family],
            Promise => &[Grief],
            SeasonTurns => &[DebtOrTrade, Threat, Family],
            _ => &[],
        }
    }
}
