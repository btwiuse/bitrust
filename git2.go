package main

import "fmt"
import "strings"
import "log"
import "io/ioutil"
import "github.com/navigaid/pretty"
import git "gopkg.in/libgit2/git2go.v27"

type commit struct {
	/// SHA1 hash
	Hash string `json:"hash"`
	/// Author name
	Author string `json:"author"`
	/// Commit creationg date as UNIX timestamp
	Date int64 `json:"date"`
	/// Commit message
	Message string `json:"message"`
}

func main() {
	repo, err := git.OpenRepository("rust")
	if err != nil {
		panic(err)
	}
	remotes, err := repo.Remotes.List()
	if err != nil {
		panic(err)
	}
	fmt.Println("remotes:", remotes)

	revwalk, err := repo.Walk()
	if err != nil {
		panic(err)
	}
	revwalk.PushHead()
	count := 0
	var commits []commit
	revwalk.Iterate(func(c *git.Commit) bool {
		if strings.Contains(c.Message(), "[breaking-change]") {
			var (
				sig     = c.Author()
				name    = sig.Name
				date    = sig.When.Unix()
				email   = sig.Email
				author  = fmt.Sprintf("%s (%s)", name, email)
				message = c.Message()
				hash    = c.AsObject().Id().String() //c.TreeId().String()
				commit  = commit{
					Author:  author,
					Date:    date,
					Message: message,
					Hash:    hash,
				}
			)
			commits = append(commits, commit)
			pretty.JSON(commit)
			println(count)
			count++
		}
		return true
	})
	// https://golang.org/pkg/io/ioutil/#example_WriteFile
	message := []byte(pretty.JSONString(commits))
	err = ioutil.WriteFile("log2.json", message, 0644)
	if err != nil {
		log.Fatal(err)
	}
	//fmt.Println(GetCommits(oid, revwalk))
	/*
		branches, err := GetBranches(repo)
		if err != nil {
			panic(err)
		}
		for i, br := range branches {
			fmt.Printf("branch[%d]=%s\n", i, br.Name)
			for j, cmt := range br.Commits {
				fmt.Println(j, cmt.ID)
			}
		}
	*/
	master, ok := GetBranchByName("master", repo)
	if !ok {
		panic(fmt.Errorf("failed to get branch master"))
	}
	var breaks []Commit
	isBreak := func(cmt Commit) bool {
		return strings.Contains(cmt.Message, "[breaking-change]")
	}
	for _, cmt := range master.Commits {
		if isBreak(cmt) {
			breaks = append(breaks, cmt)
		}
	}
	for i, cmt := range breaks {
		fmt.Println(i, cmt.ID, cmt.Message)
	}
}

// https://github.com/pratikju/servidor/blob/e8b112a4e66f0aadfb82fd56339aee7968e0cf09/commit.go

type Commit struct {
	Message    string         `json:"message"`
	ID         string         `json:"id"`
	ObjectType string         `json:"object_type"`
	Author     *git.Signature `json:"author"`
}

func GetCommits(oid *git.Oid, revWalk *git.RevWalk) []Commit {
	var commit Commit
	var commits []Commit

	err := revWalk.Push(oid)
	if err != nil {
		return commits
	}
	f := func(c *git.Commit) bool {
		commit = Commit{Message: c.Summary(), ID: c.Id().String(), ObjectType: c.Type().String(), Author: c.Author()}
		commits = append(commits, commit)
		return true
	}
	_ = revWalk.Iterate(f)
	return commits
}

// https://github.com/pratikju/servidor/blob/e8b112a4e66f0aadfb82fd56339aee7968e0cf09/branch.go

type Branch struct {
	Name    string   `json:"name"`
	IsHead  bool     `json:"isHead"`
	Commits []Commit `json:"commits"`
}

func GetBranches(repo *git.Repository) ([]Branch, error) {
	var branch Branch
	var branches []Branch

	itr, _ := repo.NewReferenceIterator()
	refs := getReferences(itr)

	revWalk, err := repo.Walk()
	if err != nil {
		log.Println(err)
		return branches, err
	}

	for i := 0; i < len(refs); i++ {
		branch = getBranch(refs[i], revWalk)
		branches = append(branches, branch)
	}

	return branches, nil
}

func getReferences(itr *git.ReferenceIterator) []*git.Reference {
	var ref *git.Reference
	var refs []*git.Reference
	var err error
	for {
		ref, err = itr.Next()
		if err != nil {
			break
		}
		refs = append(refs, ref)
	}
	return refs
}

func getBranch(ref *git.Reference, revWalk *git.RevWalk) Branch {
	var branch Branch
	b := ref.Branch()
	name, err := b.Name()
	if err != nil {
		log.Println(err)
	}
	isHead, err := b.IsHead()
	if err != nil {
		log.Println(err)
	}
	commits := GetCommits(ref.Target(), revWalk)
	branch = Branch{Name: name, IsHead: isHead, Commits: commits}
	return branch
}

func GetBranchByName(name string, repo *git.Repository) (Branch, bool) {
	var branch Branch
	gitBranch, err := repo.LookupBranch(name, git.BranchLocal)
	if err != nil {
		return branch, false
	}

	revWalk, err := repo.Walk()
	if err != nil {
		log.Println(err)
		return branch, false
	}
	return getBranch(gitBranch.Reference, revWalk), true
}
