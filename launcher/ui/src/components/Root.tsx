import * as React from "react";

export interface Props { }
export interface State {
    running_processes: server.RunningProcess[];
    running_builds: server.RunningBuild[];
    finished_builds: server.FinishedBuild[];
    projects: server.Project[];
    errors: server.Error[];
    open_uuids: string[],
    socket: WebSocket | null,
}

export class Root extends React.Component<Props, State> {
    constructor(props: Props, context?: any) {
        super(props, context);
        const socket = this.start_websocket();
        this.state = {
            running_processes: [],
            running_builds: [],
            finished_builds: [],
            projects: [],
            errors: [],
            open_uuids: [],
            socket,
        };
    }

    start_websocket(): WebSocket {
        const socket = new WebSocket("ws://" + document.location.host + "/ws");
        socket.onclose = this.ws_close.bind(this);
        socket.onopen = this.ws_open.bind(this);
        socket.onmessage = this.ws_message.bind(this);
        socket.onerror = this.ws_error.bind(this);
        return socket;
    }

    ws_close(ev: CloseEvent) {
        console.log("Websocket closed, reconnecting in 5 secs", ev);
        this.setState({ socket: null });
        setTimeout(() => {
            if (this.state.socket === null) {
                const socket = this.start_websocket();
                this.setState({ socket });
            }
        }, 5000);
    }

    ws_error(ev: Event) {
        console.log("Websocket error", ev);
    }

    ws_open(ev: Event) {
        console.log("Websocket opened");
    }

    ws_message(ev: MessageEvent) {
        const json = JSON.parse(ev.data);
        if (Array.isArray(json.running_processes)) {
            this.setState(json);
            return;
        }
        const change = json as server.ChangeState;
        if (change.ErrorAdded) {
            let errors = this.state.errors;
            errors.splice(0, 0, change.ErrorAdded);
            this.setState({ errors });
        } else if (change.ProjectsSet) {
            this.setState({
                projects: change.ProjectsSet,
            });

            // Processes
        } else if (change.RunningProcessAdded) {
            let running_processes = this.state.running_processes;
            running_processes.push(change.RunningProcessAdded);
            this.setState({ running_processes });
        } else if (change.RunningProcessRemoved) {
            let running_processes = this.state.running_processes;
            let index = running_processes.findIndex(p => p.uuid == change.RunningProcessRemoved);
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes });
            }
        } else if (change.RunningProcessStdout) {
            let running_processes = this.state.running_processes;
            let index = running_processes.findIndex(p => p.uuid == change.RunningProcessStdout![0]);
            if (index !== null) {
                running_processes[index].stdout += change.RunningProcessStdout[1];
                this.setState({ running_processes });
            }
        } else if (change.RunningProcessStderr) {
            let running_processes = this.state.running_processes;
            let index = running_processes.findIndex(p => p.uuid == change.RunningProcessStderr![0]);
            if (index !== null) {
                running_processes[index].stderr += change.RunningProcessStderr[1];
                this.setState({ running_processes });
            }
        } else if (change.RunningProcessTerminated) {
            let running_processes = this.state.running_processes;
            let index = running_processes.findIndex(p => p.uuid == change.RunningProcessTerminated![0]);
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes });
            }
        } else if (change.RunningProcessFinished) {
            let running_processes = this.state.running_processes;
            let index = running_processes.findIndex(p => p.uuid == change.RunningProcessFinished![0]);
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes });
            }

            // Builds
        } else if (change.RunningBuildAdded) {
            let running_builds = this.state.running_builds;
            running_builds.push(change.RunningBuildAdded);
            this.setState({ running_builds });
        } else if (change.RunningBuildStdout) {
            let running_builds = this.state.running_builds;
            let index = running_builds.findIndex(b => b.uuid == change.RunningBuildStdout![0]);
            if (index !== null) {
                running_builds[index].stdout += change.RunningBuildStdout[1];
                this.setState({ running_builds });
            }
        } else if (change.RunningBuildStderr) {
            let running_builds = this.state.running_builds;
            let index = running_builds.findIndex(b => b.uuid == change.RunningBuildStderr![0]);
            if (index !== null) {
                running_builds[index].stderr += change.RunningBuildStderr[1];
                this.setState({ running_builds });
            }
        } else if (change.RunningBuildTerminated) {
            let running_builds = this.state.running_builds;
            let index = running_builds.findIndex(b => b.uuid == change.RunningBuildTerminated![0]);
            if (index !== null) {
                let running_build = running_builds.splice(index, 1)[0];
                let finished_builds = this.state.finished_builds;

                let finished_build: server.FinishedBuild = {
                    ended_on: new Date().toISOString(),
                    status: -1,
                    error: change.RunningBuildTerminated[1],
                    ...running_build
                };
                finished_builds.splice(0, 0, finished_build);
                this.setState({ running_builds, finished_builds });
            }
        } else if (change.RunningBuildFinished) {
            let running_builds = this.state.running_builds;
            let index = running_builds.findIndex(b => b.uuid == change.RunningBuildFinished![0]);
            if (index !== null) {
                let running_build = running_builds.splice(index, 1)[0];
                let finished_builds = this.state.finished_builds;

                let finished_build: server.FinishedBuild = {
                    ended_on: new Date().toISOString(),
                    status: change.RunningBuildFinished[1],
                    error: null,
                    ...running_build
                };
                finished_builds.splice(0, 0, finished_build);
                this.setState({ running_builds, finished_builds });
            }
        } else {
            console.log("Unknown server command", change);
        }
    }

    render_time(diff: number) {
        diff = Math.ceil(diff / 1000);
        let result = "";
        let show_seconds = true;
        let show_minutes = true;
        let show_hours = true;
        if (diff >= 86400) {
            let days = Math.floor(diff / 86400);
            diff -= days * 3600;
            result += days + " days";
            show_minutes = false;
            show_seconds = false;
        }
        if (diff >= 3600 && show_hours) {
            let hours = Math.floor(diff / 3600);
            diff -= hours * 3600;
            result += hours + " hours";
            show_seconds = false;
        }
        if (diff >= 60 && show_minutes) {
            if (result) result += ", ";
            let minutes = Math.floor(diff / 60);
            diff -= minutes * 60;
            result += minutes + " minutes";
        }
        if (diff > 0 && show_seconds) {
            if (result) result += ", ";
            result += diff + " seconds";
        }
        return result;
    }

    render_running_build(build: server.RunningBuild, index: number): JSX.Element {
        let start = new Date(build.started_on);
        let diff = Date.now() - start.getTime();
        return <div key={index}>
            <p onClick={this.toggle_open.bind(this, build.uuid)}>
                <b>{build.directory}::{build.build}</b> (running for {this.render_time(diff)})
            </p>
            <pre>{build.stdout}</pre>
            <pre>{build.stderr}</pre>
        </div>;
    }

    render_finished_build(build: server.FinishedBuild, index: number): JSX.Element {
        let start = new Date(build.started_on);
        let end = new Date(build.ended_on);
        let diff = end.getTime() - start.getTime();
        let is_open = this.state.open_uuids.some(u => u == build.uuid);

        let status_text, status_color;
        if (build.error || build.status !== 0) {
            status_text = build.error || "Error";
            status_color = "red";
        } else {
            status_text = "Success";
            status_color = "green";
        }
        let title = <p onClick={this.toggle_open.bind(this, build.uuid)} key={index}>
            <b>{build.directory}::{build.build}</b>
            {' '}
            <b style={{ color: status_color }}>{status_text}</b>
            {' '}
            (finished {this.render_time(Date.now() - end.getTime())} ago, in {this.render_time(diff)})
        </p>;
        if (!is_open) {
            return title;
        }
        return <div key={index}>
            {title}
            <pre>{build.stdout}</pre>
            <pre>{build.stderr}</pre>
        </div>;
    }

    render_process(process: server.RunningProcess, index: number) {
        let is_open = this.state.open_uuids.some(u => u == process.uuid);
        if (is_open) {
            return <div key={index}>
                <p onClick={this.toggle_open.bind(this, process.uuid)}>
                    <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.pid)}>&times;</a>
                </p>
                <pre>{process.stdout}</pre>
                <pre>{process.stderr}</pre>
            </div>;
        } else {
            return <p key={index} onClick={this.toggle_open.bind(this, process.uuid)}>
                <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.pid)}>&times;</a>
            </p>;
        }
    }

    kill_process(id: number, ev: React.MouseEvent<HTMLAnchorElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        if (this.state.socket) {
            console.log(JSON.stringify({
                kill: id
            }));
            this.state.socket.send(JSON.stringify({
                kill: id
            }));
        }

        return false;
    }
    render_project(project: server.Project, index: number) {
        return <p key={index}><b>{project.name}</b> {project.builds.map(this.render_build.bind(this, project))}</p>;
    }
    render_build(project: server.Project, build: server.Build, index: number) {
        return <button key={index} onClick={this.start_build.bind(this, project, build)}>{build.name}</button>;
    }
    toggle_open(uuid: string, ev: React.MouseEvent<HTMLElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        let uuids = this.state.open_uuids;
        let index = uuids.findIndex(u => u == uuid);
        if (index !== null && index >= 0) {
            uuids.splice(index, 1);
        } else {
            uuids.push(uuid);
        }
        this.setState({
            open_uuids: uuids,
        });

        return false;
    }
    start_build(project: server.Project, build: server.Build, ev: React.MouseEvent<HTMLButtonElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        if (this.state.socket) {
            this.state.socket.send(JSON.stringify({
                start: [project.name, build.name]
            }));
        }

        return false;
    }

    render_error(err: server.Error, index: number) {
        return <p key={index}>
            <b>{this.render_time(Date.now() - new Date(err.time).getTime())} ago</b><br />
            {err.error}
        </p>;
    }

    render() {
        return <>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'row', borderBottom: '1px solid #555' }}>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    <h2>Processes:</h2>
                    {this.state.running_processes.map(this.render_process.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    {this.state.errors.map(this.render_error.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", padding: 5 }}>
                    {this.state.projects.map(this.render_project.bind(this))}
                </div>
            </div>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'row' }}>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    <h2>Running:</h2>
                    {this.state.running_builds.map(this.render_running_build.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", padding: 5 }}>
                    <h2>Finished:</h2>
                    {this.state.finished_builds.map(this.render_finished_build.bind(this))}
                </div>
            </div>
        </>;
    }
}