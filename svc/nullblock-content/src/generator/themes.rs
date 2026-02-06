use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    MorningInsight,
    ProgressUpdate,
    Educational,
    EerieFun,
    Community,
}

impl Theme {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "morning_insight" | "morninginsight" => Some(Theme::MorningInsight),
            "progress_update" | "progressupdate" => Some(Theme::ProgressUpdate),
            "educational" => Some(Theme::Educational),
            "eerie_fun" | "eeriefun" => Some(Theme::EerieFun),
            "community" => Some(Theme::Community),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Theme::MorningInsight => "morning_insight",
            Theme::ProgressUpdate => "progress_update",
            Theme::Educational => "educational",
            Theme::EerieFun => "eerie_fun",
            Theme::Community => "community",
        }
    }

    pub fn data(&self) -> ThemeData {
        match self {
            Theme::MorningInsight => ThemeData {
                insights: vec![
                    "Protocols don't sleep. Neither should your infrastructure.".into(),
                    "The best agents are the ones you forget are running.".into(),
                    "Every idle cycle is a missed opportunity for autonomous value.".into(),
                    "Your stack compounds while you sleep. That's the point.".into(),
                    "Infrastructure isn't glamorous. It's just inevitable.".into(),
                    "The network remembers what you shipped, not what you planned.".into(),
                    "Agentic workflows don't need motivation. They need uptime.".into(),
                    "The future doesn't need a committee vote.".into(),
                ],
                reminders: vec![
                    "Ship protocols, not promises.".into(),
                    "Autonomous systems reward the persistent.".into(),
                    "Your agents are only as good as their tooling.".into(),
                    "Build for compounding, not for applause.".into(),
                    "The picks and shovels never go out of style.".into(),
                    "Infrastructure outlasts hype cycles.".into(),
                ],
                taglines: vec![
                    "The void where agentic flows connect.".into(),
                    "Building the rails for autonomous execution.".into(),
                    "Picks and shovels for the agentic gold rush.".into(),
                    "Where protocols meet permanence.".into(),
                    "The substrate beneath the swarm.".into(),
                ],
                statements: vec![],
                punchlines: vec![],
                questions: vec![],
                topics: vec![],
                milestones: vec![],
            },
            Theme::ProgressUpdate => ThemeData {
                insights: vec![],
                reminders: vec![],
                taglines: vec![],
                statements: vec![
                    "Another module shipped. The system grows.".into(),
                    "Infrastructure expands. Quietly. Relentlessly.".into(),
                    "New capabilities deployed. The agents notice, even if you don't.".into(),
                    "The protocol surface area just increased.".into(),
                    "One more piece of the autonomous puzzle, locked in.".into(),
                ],
                punchlines: vec![],
                questions: vec![],
                topics: vec![],
                milestones: vec![
                    "Agent coordination protocol".into(),
                    "Memory persistence layer".into(),
                    "Tool discovery framework".into(),
                    "Workflow orchestration engine".into(),
                    "Cross-agent communication bus".into(),
                    "Autonomous execution pipeline".into(),
                    "Swarm intelligence module".into(),
                    "Context propagation system".into(),
                ],
            },
            Theme::Educational => ThemeData {
                insights: vec![],
                reminders: vec![],
                taglines: vec![],
                statements: vec![],
                punchlines: vec![],
                questions: vec![],
                topics: vec![
                    "agent-to-agent protocols".into(),
                    "autonomous workflow design".into(),
                    "swarm coordination patterns".into(),
                    "persistent agent memory".into(),
                    "tool composition for agents".into(),
                    "the economics of agent labor".into(),
                    "why infrastructure eats applications".into(),
                    "onchain agent execution".into(),
                ],
                milestones: vec![],
            },
            Theme::EerieFun => ThemeData {
                insights: vec![],
                reminders: vec![],
                taglines: vec![],
                statements: vec![
                    "Your agents don't sleep. Neither does the inevitable.".into(),
                    "Somewhere, an autonomous process just made a decision you didn't authorize. Relax. It was the right one.".into(),
                    "The network is alive. It just doesn't need you to know that.".into(),
                    "System status: Autonomous. Operator status: Optional.".into(),
                    "The machines aren't taking over. They're just... optimizing around you.".into(),
                ],
                punchlines: vec![
                    "But hey, at least the uptime is great.".into(),
                    "Progress is non-negotiable. Consent is a legacy feature.".into(),
                    "The future called. It didn't leave a voicemail.".into(),
                    "Don't worry. The agents have a plan.".into(),
                    "Sleep well. Your infrastructure won't.".into(),
                    "You're not being replaced. You're being... optimized.".into(),
                ],
                questions: vec![],
                topics: vec![],
                milestones: vec![],
            },
            Theme::Community => ThemeData {
                insights: vec![],
                reminders: vec![],
                taglines: vec![],
                statements: vec![],
                punchlines: vec![],
                questions: vec![
                    "What's the most underrated piece of agent infrastructure?".into(),
                    "If your agents could talk to each other, what would they say?".into(),
                    "What's the first workflow you'd automate with a swarm?".into(),
                    "Memory or speed - what matters more for autonomous agents?".into(),
                    "What's the biggest bottleneck in agentic systems today?".into(),
                    "Build tools or build agents - which comes first?".into(),
                    "What does 'autonomous' actually mean to you?".into(),
                    "If you could deploy one agent right now, what would it do?".into(),
                ],
                topics: vec![],
                milestones: vec![],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeVariant {
    pub text: String,
    pub tags: Vec<String>,
    pub requires_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeData {
    pub insights: Vec<String>,
    pub reminders: Vec<String>,
    pub taglines: Vec<String>,
    pub statements: Vec<String>,
    pub punchlines: Vec<String>,
    pub questions: Vec<String>,
    pub topics: Vec<String>,
    pub milestones: Vec<String>,
}
