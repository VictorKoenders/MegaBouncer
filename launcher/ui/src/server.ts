declare module server {

    export interface RunningProcess {
        uuid: string;
        directory: string;
        run_type: string;
        id: number;
        stdout: string;
        stderr: string;
    }

    export interface RunningBuild {
        uuid: string;
        directory: string;
        build: string;
        started_on: string;
        id: number;
        stdout: string;
        stderr: string;
    }

    export interface FinishedBuild {
        uuid: string;
        directory: string;
        build: string;
        started_on: any;
        ended_on: string;
        status: number;
        error: string | null;
        id: number;
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
        running_processes: RunningProcess[];
        running_builds: RunningBuild[];
        finished_builds: FinishedBuild[];
        projects: Project[];
        errors: Error[];
    }

}


