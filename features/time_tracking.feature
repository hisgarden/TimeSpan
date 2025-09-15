Feature: Time Tracking
  As a developer
  I want to track time spent on different projects
  So that I can monitor my productivity and bill clients accurately

  Background:
  Given a clean timespan database
    And I have a project called "My Project"

  Scenario: Start tracking time for a project
    When I start tracking time for project "My Project"
    Then the timer should be running
    And the current project should be "My Project"

  Scenario: Stop tracking time
    Given I am tracking time for project "My Project"
    When I stop tracking time
    Then the timer should be stopped
    And a time entry should be created for "My Project"

  Scenario: Start tracking with a task description
    When I start tracking time for project "My Project" with task "Fix bug #123"
    Then the timer should be running
    And the current task should be "Fix bug #123"

  Scenario: Cannot start tracking when already tracking
    Given I am tracking time for project "My Project"
    When I attempt to start tracking time for project "Another Project"
    Then I should get an error "Already tracking time"
    And the current project should still be "My Project"

  Scenario: Get current status when not tracking
    When I check the current status
    Then I should see "No active timer"

  Scenario: Get current status when tracking
    Given I am tracking time for project "My Project"
    When I check the current status  
    Then I should see the elapsed time
    And I should see "My Project" as the current project