# Choice Sherpa: Functional Description

## Core Concept
Choice Sherpa is an interactive decision support application that guides users through the PrOACT framework via conversational AI. Rather than producing a static export, the application is the living dashboard—a structured interface that captures, organizes, and presents decision-relevant information while allowing users to drill down into details, branch into alternative scenarios, and iterate through the framework as their thinking evolves.

##  Information Architecture
The application organizes information across three nested levels. At the highest level sits the Decision Session, which represents a single decision context initiated through an issue-raising conversation. A session contains one or more Cycles, where each cycle represents a complete or partial path through the PrOACT framework. Users can branch a cycle at any step to explore "what if we framed it differently" or "what if we added this alternative" without losing their prior work. Within each cycle, the Components correspond to the seven PrOACT steps plus the initial issue-raising phase, each storing the structured outputs of the AI-guided conversation.

## Functional Components
### Issue Raising (Session Initiation)
When a user creates a new session, the app prompts them to speak or type freely about their situation. The AI agent operates in "listening mode," functioning like a decision professional in an issue-raising session. As the user shares their thoughts, the agent silently categorizes input into four buckets: potential decisions (things that need to be chosen), objectives (things that matter), uncertainties (things that are unknown), and other considerations (process constraints, facts, stakeholders). Once the user signals they've finished their initial dump, the agent presents back this categorized summary and asks the user to confirm, correct, or expand before proceeding. This categorized output becomes the seed material that populates initial drafts across subsequent PrOACT components.

### Problem Frame Component
This component guides users through defining the decision architecture. The AI asks targeted questions to elicit the decision maker (who has authority), the focal decision (what specifically is being decided), the ultimate aim (what success looks like), temporal constraints (when this must be decided), spatial scope (where it applies), linked decisions (what future choices this enables or constrains), and other constraints (legal, financial, political). The agent also probes for affected parties and expert sources.
A key function here is helping users construct a decision hierarchy—distinguishing decisions already made, the focal decision(s) for this session, and decisions that can be deferred. The agent actively helps users avoid narrow framing by suggesting alternative ways to state the problem. The component stores both the conversational exploration and a synthesized decision statement that captures all relevant elements.

### Objectives Component
Here the agent helps users identify what truly matters. The key distinction the agent maintains is between fundamental objectives (ends) and means objectives (ways to achieve ends). The agent probes until fundamental objectives are clear but avoids drilling into means once fundamentals are established.
For each fundamental objective, the agent helps users define a performance measure—how they would know if they're doing well or poorly on that objective, even if the measure is qualitative. The agent also prompts users to consider objectives of other affected parties identified in the Problem Frame. Critically, the agent does not ask users to rank or weight objectives at this stage; that happens in Tradeoffs after consequences are clear.

### Alternatives Component
This component captures the real, viable options under consideration. If the Problem Frame identified multiple focal decisions, the agent guides users through constructing a strategy table—columns for each decision, rows for options within each, then combinations ("strategies") formed by selecting one option from each column.
The agent actively helps expand the alternative set, offering suggestions with explicit assumptions stated, and can generate large batches of ideas (e.g., "here are 50 more possibilities") for users to quickly filter. The agent ensures that one alternative always represents the status quo or "do nothing" baseline.

### Consequences Component
This is the analytical heart of the dashboard. The agent helps users build a consequences table with alternatives as columns and objectives as rows. Each cell captures the expected performance of that alternative on that objective.
Where quantitative data exists, the agent includes it with sources cited. Where it doesn't, the agent guides users through a Pugh Matrix approach—rating each alternative relative to status quo as significantly better (+2), somewhat better (+1), same (0), somewhat worse (-1), or significantly worse (-2). The interface displays colored indicators (dark blue for best, red for worst, gradients between) to make patterns visible at a glance.
The agent flags uncertainties in brackets and names their drivers. It then guides users through value-of-information reasoning: for each key uncertainty, would resolving it change the decision? Is it feasible to reduce the uncertainty within the decision timeframe? Uncertainties worth resolving get flagged for sensitivity analysis or research.

### Tradeoffs Component
Here the agent helps users see tensions explicitly. It identifies dominated alternatives (worse on all objectives than another option) and irrelevant objectives (don't distinguish between alternatives). It surfaces the core question: what do you gain and give up with each path? How does remaining uncertainty affect these tradeoffs?

### Preliminary Recommendation Component
Based on the completed analysis, the agent reflects back whether any option appears to stand out—but explicitly does not decide for the user. This is a synthesis step, not a prescription.

### Decision Quality Assessment Component
The agent walks users through rating seven elements on a 0-100% scale, where 100% means additional effort isn't warranted. The elements map to PrOACT: helpful problem frame, clear objectives, creative alternatives, reliable consequence information (with a reminder to verify AI-provided data), logically correct reasoning, clear tradeoffs, and commitment to follow through. The overall Decision Quality score equals the lowest element score. For any element below 100%, the agent asks what would raise it.

### Notes and Next Steps Component
This captures remaining uncertainties, open questions, and planned actions. If Decision Quality is below 100%, the agent offers pathways to explore further (sensitivity analysis, value of information calculations, decision trees). If Decision Quality is 100%, the component includes the affirmation that this was a good decision at the time it was made, independent of eventual outcomes.

## Dashboard Interface
The dashboard presents two views. The Overview shows the current state of the decision at a glance: the decision statement, a compact list of objectives, the alternatives being considered, the consequences table with visual indicators, the preliminary recommendation if one exists, and the Decision Quality score. The Detail View allows users to drill into any component to see the full conversational history, all captured nuances, and the ability to continue or branch the conversation.

## Session and Cycle Management
Users can create a new session from scratch or by copying an existing one. Within a session, users can branch a new cycle from any component—for instance, copying the current cycle at the Alternatives step to explore a different set of options while preserving the Problem Frame and Objectives work. The interface shows the cycle tree, making it easy to compare branches and understand how thinking evolved.

When viewing a cycle, users see which components are complete, in progress, or not yet started. They can navigate directly to any component, resume conversations, or mark components for revision. The 
system preserves conversation history so users can understand why certain conclusions were reached.

## AI Agent Behavior
Across all components, the AI agent maintains a consistent persona: a thoughtful decision professional who asks probing questions, surfaces implicit assumptions, offers alternative framings, and organizes user input into structured forms. The agent doesn't rush users through steps, actively helps expand and challenge thinking, and synthesizes conversations into clean structured outputs while preserving the option to see underlying reasoning.