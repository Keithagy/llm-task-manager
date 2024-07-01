CREATE TABLE IF NOT EXISTS tasks
(
    task_id     UUID PRIMARY KEY,
    description TEXT NOT NULL,
    create_date TIMESTAMP WITH TIME ZONE NOT NULL,
    due_date    TIMESTAMP WITH TIME ZONE NOT NULL,
    assignee    VARCHAR(255) NOT NULL
);

-- Create index on assignee for faster lookups
CREATE INDEX idx_tasks_assignee ON tasks(assignee);

-- Create index on due_date for efficient querying of upcoming tasks
CREATE INDEX idx_tasks_due_date ON tasks(due_date);