import * as React from "react";

export interface Props { }
export interface State {
    state: server.State | null,
    open_uuids: string[],
}

export class Root extends React.Component<Props, State> {
    interval: number;

    constructor(props: Props, context?: any) {
        super(props, context);
        this.state = {
            state: null,
            open_uuids: [],
        };
        this.interval = 0;
    }

    componentWillMount() {
        this.fetch();
    }

    fetch() {
        fetch("/api/state")
            .then(r => r.json())
            .then((r: server.State) => {
                if (this.state.state) {
                    let running_frontend_build = this.state.state.running_builds.find(b => b.directory == "launcher" && b.build == "webpack");
                    if (running_frontend_build) {
                        let finished_build = r.finished_builds.find(b => b.uuid == running_frontend_build!.uuid);
                        if (finished_build && finished_build.error === null && finished_build.status === 0) {
                            document.location.reload();
                        }
                    }
                }
                this.setState({
                    state: r
                });
                clearTimeout(this.interval);
                this.interval = setTimeout(this.fetch.bind(this), 1000);
            })
            .catch(e => {
                console.error(e);
                clearTimeout(this.interval);
                this.interval = setTimeout(this.fetch.bind(this), 1000);
            });
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
            console.log(build);
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
                    <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.id)}>&times;</a>
                </p>
                <pre>{process.stdout}</pre>
                <pre>{process.stderr}</pre>
            </div>;
        } else {
            return <p key={index} onClick={this.toggle_open.bind(this, process.uuid)}>
                <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.id)}>&times;</a>
            </p>;
        }
    }

    kill_process(id: number, ev: React.MouseEvent<HTMLAnchorElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        fetch("/api/kill/" + id).then(r => r.text()).then(r => {
            if (r !== "Ok") {
                alert("Could not kill process\n" + r);
            }
        });

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

        fetch("/api/build/start/" + project.name + "/" + build.name).then(r => r.text()).then(t => {
            if (t != "Ok") {
                alert("Could not start build\n" + t);
            }
        });

        return false;
    }

    render_error(err: server.Error, index: number) {
        return <p key={index}>
            <b>{this.render_time(Date.now() - new Date(err.time).getTime())} ago</b><br />
            {err.error}
        </p>;
    }

    render() {
        if (!this.state.state) return <></>;
        return <>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'row', borderBottom: '1px solid #555' }}>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    <h2>Processes:</h2>
                    {this.state.state.running_processes.map(this.render_process.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    {this.state.state.errors.map(this.render_error.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", padding: 5 }}>
                    {this.state.state.projects.map(this.render_project.bind(this))}
                </div>
            </div>
            <div style={{ flex: 1, display: 'flex', flexDirection: 'row' }}>
                <div style={{ flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 }}>
                    <h2>Running:</h2>
                    {this.state.state.running_builds.map(this.render_running_build.bind(this))}
                </div>
                <div style={{ flex: 1, overflow: "auto", padding: 5 }}>
                    <h2>Finished:</h2>
                    {this.state.state.finished_builds.map(this.render_finished_build.bind(this))}
                </div>
            </div>
        </>;
    }
}