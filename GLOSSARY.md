# Flowrs

A terminal interface for browsing and operating Apache Airflow: the user picks an Airflow environment and moves down from DAGs to runs to task instances to logs.

## Airflow

**Environment**:
One Airflow deployment the user has configured or discovered through a managed service.
_Avoid_: Server, config, instance

**DAG**:
A workflow definition in an environment, identified by its dag id.
_Avoid_: Pipeline, job

**DAG run**:
One execution of a DAG, identified by its run id.
_Avoid_: Run instance, execution

**Task instance**:
One task within a DAG run.
_Avoid_: Task run, job

**Try**:
One attempt at running a task instance; a task instance has one try per retry, each with its own log.
_Avoid_: Attempt, retry number

## Navigation

**Path**:
What the user has selected, from the outside in: an environment, then a DAG, then a DAG run, then a task instance. Each part exists only if the part above it does.
_Avoid_: Context, navigation context, selection

**Level**:
Which part of the path the user is looking at: Environments, DAGs, DAG runs, Task instances or Logs. The level never goes deeper than the path.
_Avoid_: Panel, screen, view
