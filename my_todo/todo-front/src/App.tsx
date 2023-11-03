import { FC, useState } from 'react'
import 'modern-css-reset'
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Box, Stack, Typography } from '@mui/material';
import { NewTaskPayload, Task } from './types/task';
import TaskList from './components/TaskList.tsx';
import TaskForm from './components/TaskForm.tsx';

const TodoApp: FC = () => {
    const [tasks, setTasks] = useState<Task[]>([]);
    const createId = () => tasks.length + 1;

    const onSubmit = async (payload: NewTaskPayload) => {
        if (!payload.text) {
            return;
        }
        setTasks((prev) => [
            {
                id: createId(),
                text: payload.text,
                completed: false,
            },
            ...prev,
        ]);
    };

    const onUpdate = (updateTask: Task) => {
        setTasks(
            tasks.map((task) => {
                if (task.id === updateTask.id) {
                    return {
                        ...task,
                        ...updateTask, // 必要な部分だけ overwrite
                    };
                }
                return task;
            })
        );
    };

    return (
        <>
            <Box
                sx={{
                    backgroundColor: 'white',
                    borderBottom: '1px solid gray',
                    display: 'flex',
                    alignItem: 'center',
                    position: 'fixed',
                    top: 0,
                    p: 2,
                    width: '100%',
                    height: 80,
                    zIndex: 3,
                }}
            >
                <Typography variant="h1">Todo App</Typography>
            </Box>
            <Box
                sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    p: 5,
                    mt: 10,
                }}
            >
                <Box maxWidth={700} width="100%">
                    <Stack spacing={5}>
                        <TaskForm onSubmit={onSubmit} />
                        <TaskList tasks={tasks} onUpdate={onUpdate} />
                    </Stack>
                </Box>
            </Box>
        </>
    )
};

const theme = createTheme({
    typography: {
        h1: {
            fontSize: 30,
        },
        h2: {
            fontSize: 20,
        },
    },
});

const App: FC = () => {
    return (
        <ThemeProvider theme={theme}>
            <TodoApp />
        </ThemeProvider>
    );
}

export default App
