Feature: Time Reporting
  As a developer
  I want to generate reports of my time tracking
  So that I can analyze my productivity and create invoices

  Background:
    Given a clean toggl database
    And I have a project called "Project A"
    And I have a project called "Project B"

  Scenario: Generate daily report
    Given I have time entries for today:
      | project   | duration | task          |
      | Project A | 2h 30m   | Development   |
      | Project B | 1h 15m   | Code review   |
      | Project A | 45m      | Testing       |
    When I generate a daily report for today
    Then I should see total time of "4h 30m"
    And I should see "Project A: 3h 15m"
    And I should see "Project B: 1h 15m"

  Scenario: Generate weekly report
    Given I have time entries for this week:
      | project   | day       | duration |
      | Project A | Monday    | 4h       |
      | Project A | Tuesday   | 3h 30m   |
      | Project B | Wednesday | 2h       |
    When I generate a weekly report
    Then I should see total time of "9h 30m"
    And I should see "Project A: 7h 30m"
    And I should see "Project B: 2h"

  Scenario: Export report as JSON
    Given I have time entries for today:
      | project   | duration | task        |
      | Project A | 2h       | Development |
    When I export today's report as JSON
    Then I should get a valid JSON response
    And the JSON should contain project "Project A"
    And the JSON should contain duration "2h"

  Scenario: Generate report with no entries
    When I generate a daily report for today
    Then I should see "No time entries found"
    And total time should be "0h 0m"

  Scenario: Filter report by project
    Given I have time entries for today:
      | project   | duration |
      | Project A | 2h       |
      | Project B | 1h       |
    When I generate a report filtered by project "Project A"
    Then I should see "Project A: 2h"
    And I should not see "Project B"