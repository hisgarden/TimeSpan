Feature: Project Management
  As a developer
  I want to manage my projects
  So that I can organize my time tracking

  Background:
  Given a clean timespan database

  Scenario: Create a new project
    When I create a project called "New Project"
    Then the project "New Project" should exist
    And the project should have a unique ID

  Scenario: List all projects
    Given I have projects:
      | name           | description        |
      | Project Alpha  | First project      |
      | Project Beta   | Second project     |
    When I list all projects
    Then I should see "Project Alpha"
    And I should see "Project Beta"

  Scenario: Create project with description
    When I create a project called "Client Work" with description "Work for ACME Corp"
    Then the project "Client Work" should have description "Work for ACME Corp"

  Scenario: Cannot create duplicate projects
    Given I have a project called "Existing Project"
    When I attempt to create a project called "Existing Project"
    Then I should get an error "Project already exists"

  Scenario: Delete a project
    Given I have a project called "Old Project"
    When I delete the project "Old Project"
    Then the project "Old Project" should not exist

  Scenario: Cannot delete a project with time entries
    Given I have a project called "Active Project"
    And I have time entries for "Active Project"
    When I attempt to delete the project "Active Project"
    Then I should get an error "Cannot delete project with time entries"