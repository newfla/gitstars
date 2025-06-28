import React, { useState } from 'react';
import { FaGithub, FaGitlab, FaHeart, FaRegHeart, FaStar } from 'react-icons/fa';
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Ok<T> {
  Ok: T,
}

interface Error {
  Err: string,
}

type Result<T> = Ok<T> | Error

enum GitProvider {
  GitHub = "GitHub",
  GitLab = "GitLab",
}

interface Repo {
  git_type: GitProvider,
  owner: string,
  name: string,
}

interface Setting {
  id: string,
  order: number,
  favourite: boolean,
  repo: Repo,
}

interface Fetched {
  setting: Setting,
  stars: number,
}

//Init data 
const read: Result<Fetched>[] = await invoke("read");
console.log(read)
const data: Fetched[] = read.filter((data) => data && "Ok" in data).map((data) => { return data.Ok })

export default function RepoList() {

  const [repos, setRepos] = useState<Fetched[]>(data);
  const [repoSelectedProvider, setRepoSelectedProvider] = useState(GitProvider.GitHub);
  const [repoName, setRepoName] = useState('');
  const [repoOwner, setRepoOwner] = useState('');


  const addRepo = async function () {
    if (repoName && repoOwner) {
      const repo: Repo = {
        git_type: repoSelectedProvider,
        owner: repoOwner,
        name: repoName,
      };
      const id: string = await invoke("uuid");
      const setting: Setting = {
        id,
        order: repos.length,
        favourite: false,
        repo
      }
      try {
        const stars: number = await invoke("create", { setting })
        if (typeof stars === "number") {
          const fetched: Fetched = {
            setting,
            stars
          }
          setRepos([...repos, fetched]);
          setRepoName('');
          setRepoOwner('');
        }
      } catch (error) {
        console.log(error)
      }
    }
  };

  const getLogo = (provider: GitProvider) => {
    switch (provider) {
      case GitProvider.GitHub:
        return <div>
          <FaGithub className="size-10 rounded-box" />
        </div>
          ;
      case GitProvider.GitLab:
        return <div><FaGitlab className="size-10 rounded-box" /></div>;
      default:
        return null;
    }
  };

  const getStarsIcon = (stars: number) => {
    return (
      <div className="flex items-center">
        <div>{stars}  </div>
        <div><FaStar className="size-5 rounded-box" /></div>
      </div>
    );
  };

  const getFavouriteIcon = (which: Fetched) => {
    return (
      <button className="btn btn-square btn-ghost"
        onClick={async () => { await modFavourite(which.setting) }}>
        {which.setting.favourite ? <FaHeart /> : <FaRegHeart />}
      </button>
    );
  };

  const modFavourite = async function (which: Setting) {
    which.favourite = !which.favourite
    const new_repos = [...repos]
    for (const f of new_repos) {
      if (f.setting.id === which.id) {
        f.setting.favourite = which.favourite
        await invoke("update", { "setting": which })
      } else if (f.setting.favourite === true && which.favourite === true) {
        f.setting.favourite = false
      }
    }
    setRepos([...new_repos])
  }

  return (

    <div className="min-h-screen p-8">
      <div className="max-w-2xl mx-auto">
        <fieldset className="fieldset bg-base-200 border-base-300 rounded-box w-xs border p-4">
          <legend className="fieldset-legend">New repo</legend>
          <div className="join">
            <select className="select join-item"
              value={repoSelectedProvider}
              onChange={(e) => setRepoSelectedProvider(GitProvider[e.target.value as keyof typeof GitProvider])}
            >
              {Object.values(GitProvider).map((provider, i) => <option value={provider} key={i}>{provider} </option>)}
            </select>
            <input type="text" className="input join-item" value={repoOwner}
              onChange={(e) => setRepoOwner(e.target.value)}
              placeholder="owner" />
            <label className="input join-item">/</label>
            <input type="text" className="input join-item" value={repoName}
              onChange={(e) => setRepoName(e.target.value)}
              placeholder="name" />
            <button className="btn btn-xs sm:btn-sm md:btn-md lg:btn-lg xl:btn-xl btn-primary join-item" onClick={addRepo}>Add</button>
          </div>
        </fieldset>
        <ul className="list bg-base-100 rounded-box shadow-md">
          {repos.map((repo, index) => (
            <li className="list-row" key={index}>
              {getLogo(repo.setting.repo.git_type)}
              <div>
                <div>{repo.setting.repo.owner}/{repo.setting.repo.name}</div>
              </div>
              {getStarsIcon(repo.stars)}
              {getFavouriteIcon(repo)}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}