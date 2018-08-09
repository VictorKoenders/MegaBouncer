declare module server {

    export interface RunningBuild {
        directory: string;
        build: string;
        id: number;
        status: string;
        stdout: string;
        stderr: string;
    }

    export interface RunningProcess {
        directory: string;
        run_type: string;
        id: number;
        status: string;
        stdout: string;
        stderr: string;
    }

    export interface Status {
        Success: number;
    }

    export interface FinishedBuild {
        directory: string;
        build: string;
        id: number;
        status: Status;
        stdout: string;
        stderr: string;
    }

    export interface TriggerBuild {
        name: string;
    }

    export interface AfterSuccess {
        Run: string;
        TriggerBuild: TriggerBuild;
    }

    export interface Build {
        name: string;
        directory: string;
        pattern: string;
        build: string;
        after_success: AfterSuccess;
    }

    export interface Project {
        name: string;
        builds: Build[];
    }

    export interface Error {
        time: string;
        error: string;
    }

    export interface State {
        running_builds: RunningBuild[];
        running_processes: RunningProcess[];
        finished_builds: FinishedBuild[];
        projects: Project[];
        errors: Error[];
    }

}
