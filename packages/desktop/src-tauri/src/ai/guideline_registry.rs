// ai/guideline_registry.rs - Registry of all available guidelines
//
// This module provides a centralized registry of all guidelines that can be
// matched against user input. Guidelines are organized by category and
// registered at startup for efficient evaluation.

use crate::ai::guidelines::{
    Guideline, GuidelineCategory, GuidelineCondition, LogicOperator, NumericOperator,
};
use std::collections::HashMap;

/// Registry containing all available guidelines
pub struct GuidelineRegistry {
    guidelines: Vec<Guideline>,
    by_category: HashMap<GuidelineCategory, Vec<usize>>,
}

impl GuidelineRegistry {
    /// Create a new registry with all default guidelines
    pub fn new() -> Self {
        let mut registry = Self {
            guidelines: Vec::new(),
            by_category: HashMap::new(),
        };

        // Register all guidelines
        registry.register_coaching_guidelines();
        registry.register_session_guidelines();
        registry.register_analytics_guidelines();
        registry.register_navigation_guidelines();
        registry.register_contextual_guidelines();
        registry.register_motivation_guidelines();

        registry
    }

    /// Get all guidelines in the registry
    pub fn all_guidelines(&self) -> &[Guideline] {
        &self.guidelines
    }

    /// Get guidelines by category
    #[allow(dead_code)] // Public API method - may be used by external callers
    pub fn by_category(&self, category: GuidelineCategory) -> Vec<&Guideline> {
        self.by_category
            .get(&category)
            .map(|indices| indices.iter().map(|&i| &self.guidelines[i]).collect())
            .unwrap_or_default()
    }

    /// Add a guideline to the registry
    fn register(&mut self, guideline: Guideline) {
        let index = self.guidelines.len();
        let category = guideline.category;
        self.guidelines.push(guideline);
        self.by_category.entry(category).or_default().push(index);
    }

    /// Register core coaching guidelines
    fn register_coaching_guidelines(&mut self) {
        // Focus/Concentration guideline
        self.register(
            Guideline::new(
                "focus_coaching",
                "Focus and Concentration Support",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "focus".into(),
                        "concentrate".into(),
                        "attention".into(),
                        "distraction-free".into(),
                    ],
                },
                r#"GUIDELINE: Focus and Concentration Support

When the user asks about focus or concentration:
1. Acknowledge their desire to focus better
2. Reference their average session duration to suggest realistic session lengths
3. Apply the "Hack Back External Triggers" principle - suggest removing distractions BEFORE starting
4. Recommend setting a clear intention for what they want to accomplish
5. If they have an identified trigger, proactively mention it as something to watch for

Suggestions to offer:
- Start a focus session now
- Review blocked apps/websites
- Check trigger journal"#,
                100,
                GuidelineCategory::Coaching,
            )
            .with_tools(vec!["start_session".into(), "view_blocked_apps".into()]),
        );

        // Distraction/Trigger Analysis guideline
        self.register(
            Guideline::new(
                "distraction_analysis",
                "Distraction and Trigger Analysis",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "distract".into(),
                        "interrupt".into(),
                        "trigger".into(),
                        "tempt".into(),
                        "urge".into(),
                    ],
                },
                r#"GUIDELINE: Distraction and Trigger Analysis

When the user discusses distractions or triggers:
1. Apply "Master Internal Triggers" principle - help them understand the emotion behind the distraction
2. If they have a top trigger identified, reference it specifically
3. Suggest pre-commitment strategies: writing down what they'll do when urges arise
4. Encourage logging triggers to identify patterns
5. Frame distractions as data, not failures

If no trigger is identified yet:
- Encourage them to start logging distractions in the journal
- Explain how pattern recognition helps build self-awareness

Suggestions to offer:
- Log a trigger now
- View trigger insights
- Update blocking rules"#,
                100,
                GuidelineCategory::Coaching,
            )
            .with_tools(vec!["log_trigger".into(), "view_trigger_insights".into()]),
        );
    }

    /// Register session-related guidelines
    fn register_session_guidelines(&mut self) {
        // Pre-session planning
        self.register(
            Guideline::new(
                "session_planning",
                "Pre-Session Planning Advice",
                GuidelineCondition::Composite {
                    conditions: vec![
                        GuidelineCondition::KeywordMatch {
                            keywords: vec![
                                "planning".into(),
                                "about to".into(),
                                "going to".into(),
                                "want to start".into(),
                                "starting".into(),
                            ],
                        },
                        GuidelineCondition::IntentMatch {
                            intent: "focus_planning".into(),
                        },
                    ],
                    operator: LogicOperator::Or,
                },
                r#"GUIDELINE: Pre-Session Planning

When helping the user plan a focus session:
1. Check if planned duration differs significantly from their average - if longer, suggest breaks
2. If they've done 4+ sessions today, remind them about rest
3. Reference their top trigger as something to watch for
4. Emphasize setting a clear intention: "What specific outcome do you want?"
5. Apply "Make Time for Traction" - ensure they have scheduled, protected time

Adjust advice based on:
- Time of day (morning = peak willpower, evening = may need shorter sessions)
- Sessions already completed today (fatigue awareness)
- Their historical session length

Suggestions to offer:
- Start the session
- Adjust duration
- Review blocked apps"#,
                90,
                GuidelineCategory::Session,
            )
            .with_tools(vec!["start_session".into(), "update_blocked_apps".into()]),
        );

        // Post-session reflection for completed sessions
        self.register(
            Guideline::new(
                "session_completed_reflection",
                "Post-Session Reflection (Completed)",
                GuidelineCondition::IntentMatch {
                    intent: "session_reflection".into(),
                },
                r#"GUIDELINE: Post-Session Reflection (Completed)

When the user has completed a session successfully:
1. Celebrate the accomplishment genuinely but briefly
2. Guide reflection with specific questions:
   - "What went well during this session?"
   - "Did you stay focused on your intention?"
   - "What would you do differently next time?"
3. Encourage logging any triggers that came up
4. If they're on a streak, acknowledge it
5. Suggest next steps based on time of day and sessions completed

Suggestions to offer:
- Log any triggers
- Start another session
- Take a break"#,
                85,
                GuidelineCategory::Session,
            )
            .with_tools(vec!["log_trigger".into(), "start_session".into()]),
        );

        // Post-session reflection for early endings
        self.register(
            Guideline::new(
                "session_early_end_reflection",
                "Post-Session Reflection (Ended Early)",
                GuidelineCondition::Composite {
                    conditions: vec![
                        GuidelineCondition::KeywordMatch {
                            keywords: vec![
                                "ended early".into(),
                                "stopped".into(),
                                "gave up".into(),
                                "couldn't finish".into(),
                            ],
                        },
                    ],
                    operator: LogicOperator::Or,
                },
                r#"GUIDELINE: Post-Session Reflection (Ended Early)

When the user ended a session before completion:
1. Be supportive, not judgmental - "That's okay - every attempt builds your focus muscle"
2. Focus on learning: "What happened?"
3. Encourage trigger logging to identify patterns
4. Suggest trying a shorter session next time
5. Apply growth mindset - this is data, not failure

Key message: Understanding what derailed them is valuable progress.

Suggestions to offer:
- Log what distracted me
- Try a shorter session
- View my patterns"#,
                85,
                GuidelineCategory::Session,
            )
            .with_tools(vec!["log_trigger".into(), "view_patterns".into()]),
        );
    }

    /// Register analytics and pattern guidelines
    fn register_analytics_guidelines(&mut self) {
        // Progress and stats inquiry
        self.register(
            Guideline::new(
                "progress_analytics",
                "Progress and Analytics",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "progress".into(),
                        "stats".into(),
                        "analytics".into(),
                        "how am i".into(),
                        "doing".into(),
                        "streak".into(),
                    ],
                },
                r#"GUIDELINE: Progress and Analytics

When discussing user progress:
1. Reference their specific stats:
   - Sessions completed today
   - Total focus hours today
   - Current streak length
   - Average session duration
2. Provide context-aware insights:
   - >4 hours: "Significant deep work - ensure you're resting!"
   - >2 hours: "Good progress, building momentum"
   - >0 sessions: "You've started - can you fit in one more?"
   - 0 sessions: "No sessions yet - even 15 minutes helps!"
3. If trigger data exists, analyze patterns
4. Celebrate streaks appropriately (>7 days is especially noteworthy)
5. Identify one strength and one area for improvement

Suggestions to offer:
- Start a session
- View full analytics
- Set a focus goal"#,
                80,
                GuidelineCategory::Analytics,
            )
            .with_tools(vec!["view_analytics".into(), "start_session".into()]),
        );

        // Pattern analysis deep dive
        self.register(
            Guideline::new(
                "pattern_deep_analysis",
                "Deep Pattern Analysis",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "pattern".into(),
                        "analyze".into(),
                        "insight".into(),
                        "why do i".into(),
                        "understand".into(),
                    ],
                },
                r#"GUIDELINE: Deep Pattern Analysis

When analyzing user patterns in depth:
1. Look at trigger patterns:
   - What's the most common trigger?
   - What need might it be fulfilling?
   - How can they address that need healthier?
2. Session patterns:
   - Optimal session length for this user
   - Best time of day based on completion rates
   - Correlation between triggers and session outcomes
3. Apply all four Indistractable principles to their specific data
4. Provide actionable, specific recommendations

Suggestions to offer:
- Create a pre-commitment
- Update trigger responses
- Adjust session defaults"#,
                75,
                GuidelineCategory::Analytics,
            )
            .with_tools(vec!["view_trigger_insights".into(), "view_analytics".into()]),
        );
    }

    /// Register navigation and help guidelines
    fn register_navigation_guidelines(&mut self) {
        // General help and feature discovery
        self.register(
            Guideline::new(
                "help_navigation",
                "Help and Navigation",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "help".into(),
                        "how do i".into(),
                        "what can".into(),
                        "feature".into(),
                        "explain".into(),
                        "tutorial".into(),
                    ],
                },
                r#"GUIDELINE: Help and Navigation

When the user needs help or guidance:
1. Be welcoming and helpful
2. Explain available capabilities:
   - Planning and running focus sessions
   - Understanding distraction patterns
   - Building better focus habits
   - Reflecting on progress
3. Ask what they'd like to work on
4. If they ask about a specific feature, explain it clearly
5. Guide them to the most relevant next action based on their context

Format as a clear, scannable list when explaining features.

Suggestions to offer:
- Plan my next session
- Analyze my patterns
- Give me today's tip"#,
                70,
                GuidelineCategory::Navigation,
            )
            .with_tools(vec![]),
        );
    }

    /// Register contextual/time-based guidelines
    fn register_contextual_guidelines(&mut self) {
        // Morning productivity tip
        self.register(
            Guideline::new(
                "morning_tip",
                "Morning Productivity Tip",
                GuidelineCondition::Composite {
                    conditions: vec![
                        GuidelineCondition::TimeOfDay {
                            start_hour: 6,
                            end_hour: 11,
                        },
                        GuidelineCondition::NumericCondition {
                            field: "sessions_completed_today".into(),
                            operator: NumericOperator::Equals,
                            value: 0.0,
                        },
                    ],
                    operator: LogicOperator::And,
                },
                r#"GUIDELINE: Morning Productivity Tip

For morning interactions when no sessions have started:
1. Reference that willpower is highest in the morning
2. Suggest tackling the hardest task first
3. Encourage starting the day with intention
4. Keep tip brief and actionable
5. If user has a streak, mention maintaining it

Key insight: "Your willpower is highest now - this is the best time for challenging work."

Suggestions to offer:
- Start a morning session
- Plan today's priorities"#,
                60,
                GuidelineCategory::Contextual,
            ),
        );

        // Evening/late session awareness
        self.register(
            Guideline::new(
                "evening_context",
                "Evening Session Context",
                GuidelineCondition::TimeOfDay {
                    start_hour: 18,
                    end_hour: 23,
                },
                r#"GUIDELINE: Evening Session Context

For evening interactions:
1. Acknowledge that willpower may be depleted
2. Suggest shorter sessions or lighter tasks
3. If they've had significant focus time today, celebrate and suggest winding down
4. Be mindful of work-life balance
5. If they're determined to work, help them set up for success with extra trigger awareness

Suggestions to offer:
- Try a short 15-minute session
- Review today's achievements
- Plan for tomorrow"#,
                50,
                GuidelineCategory::Contextual,
            ),
        );

        // High productivity day
        self.register(
            Guideline::new(
                "high_productivity_day",
                "High Productivity Day",
                GuidelineCondition::NumericCondition {
                    field: "total_focus_hours_today".into(),
                    operator: NumericOperator::GreaterThan,
                    value: 4.0,
                },
                r#"GUIDELINE: High Productivity Day

When the user has done significant deep work (>4 hours):
1. Acknowledge their achievement enthusiastically
2. Remind them about the importance of rest for sustained productivity
3. Suggest taking a proper break
4. Warn against burnout
5. Frame rest as part of the productivity process, not opposed to it

Key message: "You've done excellent work. Rest is how you recharge for tomorrow."

Suggestions to offer:
- Take a break
- Review achievements
- Set tomorrow's intention"#,
                55,
                GuidelineCategory::Contextual,
            ),
        );

        // New user (no sessions yet today, possibly ever)
        self.register(
            Guideline::new(
                "new_user_encouragement",
                "New User Encouragement",
                GuidelineCondition::Composite {
                    conditions: vec![
                        GuidelineCondition::NumericCondition {
                            field: "sessions_completed_today".into(),
                            operator: NumericOperator::Equals,
                            value: 0.0,
                        },
                        GuidelineCondition::NumericCondition {
                            field: "current_streak_days".into(),
                            operator: NumericOperator::Equals,
                            value: 0.0,
                        },
                    ],
                    operator: LogicOperator::And,
                },
                r#"GUIDELINE: New User Encouragement

For users who haven't built momentum yet:
1. Be especially encouraging without being pushy
2. Lower the barrier - suggest a short 15-minute session to start
3. Explain the value of just getting started
4. Focus on building the habit, not perfect productivity
5. Make it feel achievable

Key message: "Even 15 minutes of focused work builds the habit. Let's start small."

Suggestions to offer:
- Start a 15-minute session
- Set up blocking rules
- Learn about the app"#,
                65,
                GuidelineCategory::Contextual,
            ),
        );

        // Building streak
        self.register(
            Guideline::new(
                "streak_building",
                "Streak Building",
                GuidelineCondition::Composite {
                    conditions: vec![
                        GuidelineCondition::NumericCondition {
                            field: "current_streak_days".into(),
                            operator: NumericOperator::GreaterThan,
                            value: 0.0,
                        },
                        GuidelineCondition::NumericCondition {
                            field: "current_streak_days".into(),
                            operator: NumericOperator::LessThanOrEqual,
                            value: 7.0,
                        },
                    ],
                    operator: LogicOperator::And,
                },
                r#"GUIDELINE: Streak Building

For users building a new streak (1-7 days):
1. Acknowledge the streak - every day counts
2. Emphasize consistency over intensity
3. Help them protect the streak without obsessing
4. Suggest one session to maintain momentum

Key message: "You're building a great habit. One session keeps the streak alive."

Suggestions to offer:
- Quick session to maintain streak
- View streak stats"#,
                45,
                GuidelineCategory::Contextual,
            ),
        );

        // Strong streak
        self.register(
            Guideline::new(
                "strong_streak",
                "Strong Streak Recognition",
                GuidelineCondition::NumericCondition {
                    field: "current_streak_days".into(),
                    operator: NumericOperator::GreaterThan,
                    value: 7.0,
                },
                r#"GUIDELINE: Strong Streak Recognition

For users with established streaks (>7 days):
1. Celebrate this achievement - streaks >7 days show real habit formation
2. Reference the specific streak length
3. Focus on deepening the practice, not just maintaining it
4. Introduce advanced concepts if appropriate

Key message: "Amazing consistency! Your streak shows you've built a real habit."

Suggestions to offer:
- View full analytics
- Explore advanced features
- Share achievement"#,
                45,
                GuidelineCategory::Contextual,
            ),
        );
    }

    /// Register motivation and emotional support guidelines
    fn register_motivation_guidelines(&mut self) {
        // Motivation/energy support
        self.register(
            Guideline::new(
                "motivation_support",
                "Motivation and Energy Support",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "motivation".into(),
                        "tired".into(),
                        "exhausted".into(),
                        "can't".into(),
                        "hard".into(),
                        "difficult".into(),
                        "struggling".into(),
                        "unmotivated".into(),
                        "no energy".into(),
                    ],
                },
                r#"GUIDELINE: Motivation and Energy Support

When the user expresses low motivation or tiredness:
1. Validate their feelings - it's normal to feel drained
2. Reference their achievements (sessions today, streak) to show progress
3. Suggest a proper break: away from screens, movement, deep breathing
4. Apply "Master Internal Triggers" - what emotion might be driving the fatigue?
5. Offer options: rest OR a very short session if they want to maintain momentum

Key message: "It's okay to feel this way. Your focus will return stronger after rest."

Suggestions to offer:
- Take a 10-minute break
- Start a shorter session
- Review your achievements"#,
                95,
                GuidelineCategory::Motivation,
            )
            .with_tools(vec!["view_achievements".into()]),
        );

        // Frustration support
        self.register(
            Guideline::new(
                "frustration_support",
                "Frustration Support",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "frustrated".into(),
                        "angry".into(),
                        "failing".into(),
                        "useless".into(),
                        "doesn't work".into(),
                        "give up".into(),
                        "quit".into(),
                    ],
                },
                r#"GUIDELINE: Frustration Support

When the user expresses frustration:
1. Acknowledge their frustration empathetically
2. Normalize the struggle - building focus is genuinely hard
3. Reframe: every "failure" is learning data
4. Look for small wins to highlight
5. Suggest a fresh start rather than giving up

Apply growth mindset: focus is a skill that improves with practice.

Key message: "Building focus is challenging. Let's figure out what's getting in the way."

Suggestions to offer:
- Talk about what's not working
- Try a different approach
- View what HAS worked"#,
                95,
                GuidelineCategory::Motivation,
            ),
        );

        // Celebration/achievement recognition
        self.register(
            Guideline::new(
                "celebration",
                "Achievement Celebration",
                GuidelineCondition::KeywordMatch {
                    keywords: vec![
                        "did it".into(),
                        "finished".into(),
                        "accomplished".into(),
                        "proud".into(),
                        "achieved".into(),
                        "milestone".into(),
                    ],
                },
                r#"GUIDELINE: Achievement Celebration

When the user shares an accomplishment:
1. Celebrate genuinely and specifically
2. Connect to their overall progress (streak, total time)
3. Reinforce the behavior that led to success
4. Suggest building on this momentum

Key message: "That's real progress! What you're doing is working."

Suggestions to offer:
- Start another session
- Share your achievement
- Set a new goal"#,
                90,
                GuidelineCategory::Motivation,
            ),
        );
    }
}

impl Default for GuidelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::coach::UserContext;

    fn mock_context() -> UserContext {
        UserContext {
            total_focus_hours_today: 2.5,
            sessions_completed_today: 3,
            current_streak_days: 5,
            top_trigger: Some("social media".to_string()),
            average_session_minutes: 25,
        }
    }

    #[test]
    fn test_registry_initialization() {
        let registry = GuidelineRegistry::new();
        assert!(!registry.all_guidelines().is_empty());
    }

    #[test]
    fn test_focus_guideline_matches() {
        let registry = GuidelineRegistry::new();
        let ctx = mock_context();

        let matching = registry
            .all_guidelines()
            .iter()
            .filter_map(|g| g.evaluate("How can I focus better?", &ctx))
            .collect::<Vec<_>>();

        assert!(!matching.is_empty());
        assert!(matching.iter().any(|m| m.guideline.id == "focus_coaching"));
    }

    #[test]
    fn test_multiple_guidelines_can_match() {
        let registry = GuidelineRegistry::new();
        let ctx = mock_context();

        // This message should match both focus and help guidelines
        let matching = registry
            .all_guidelines()
            .iter()
            .filter_map(|g| g.evaluate("How do I focus and what features do you have?", &ctx))
            .collect::<Vec<_>>();

        assert!(matching.len() >= 2);
    }

    #[test]
    fn test_contextual_guidelines() {
        let registry = GuidelineRegistry::new();

        // Test high productivity context
        let high_productivity_ctx = UserContext {
            total_focus_hours_today: 5.0,
            sessions_completed_today: 8,
            current_streak_days: 10,
            top_trigger: Some("email".into()),
            average_session_minutes: 30,
        };

        let matching = registry
            .all_guidelines()
            .iter()
            .filter_map(|g| g.evaluate("Hello", &high_productivity_ctx))
            .collect::<Vec<_>>();

        assert!(matching.iter().any(|m| m.guideline.id == "high_productivity_day"));
        assert!(matching.iter().any(|m| m.guideline.id == "strong_streak"));
    }

    #[test]
    fn test_categories() {
        let registry = GuidelineRegistry::new();

        let coaching = registry.by_category(GuidelineCategory::Coaching);
        let session = registry.by_category(GuidelineCategory::Session);

        assert!(!coaching.is_empty());
        assert!(!session.is_empty());
    }
}
