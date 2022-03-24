package systest

import (
	"fmt"
	"os/exec"
	"strings"
	"testing"
	"time"

	"database/sql"

	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
)

type Cluster struct {
	controlplaneCmd *exec.Cmd
	dataplaneCmd    *exec.Cmd
}

var cluster *Cluster

func runCluster() *Cluster {
	cmd := exec.Command("./controlplane", "--config-path", "./controlplane_config.yaml")
	err := cmd.Start()
	if err != nil {
		panic(err.Error())
	}
	time.Sleep(time.Second * 3)
	dataplaneCmd := exec.Command("./dataplane", "--config_file", "./dataplane_config.yaml")
	err = dataplaneCmd.Start()
	if err != nil {
		panic(err.Error())
	}
	// wait for the cluster to initialize
	time.Sleep(time.Second * 3)
	return &Cluster{
		controlplaneCmd: cmd,
		dataplaneCmd:    dataplaneCmd,
	}
}

type Credential struct {
	UserName string
	Password string
}

var Credentials = map[string]Credential{"admin": {
	UserName: "fragrant-sun",
	Password: "d8c1e29d7e58be",
},
	"dev": {
		UserName: "divine-butterfly",
		Password: "f3535f770dadb1",
	}}

func getDB(dbName string, role string, t *testing.T) *sqlx.DB {
	cred := Credentials[role]
	psqlconn := fmt.Sprintf("host=%s port=%d user=%s password=%s dbname=%s sslmode=disable", "localhost", 8081, cred.UserName, cred.Password, dbName)
	db, err := sqlx.Open("postgres", psqlconn)
	if err != nil {
		t.Fatal(err)
	}
	err = db.Ping()
	if err != nil {
		t.Fatal(err)
	}
	return db
}

func (c *Cluster) TearDown() {
	c.dataplaneCmd.Process.Kill()
	c.controlplaneCmd.Process.Kill()
}

func TestMain(t *testing.M) {
	cluster = runCluster()
	t.Run()
	cluster.TearDown()
}

func assert(assert bool, msg string, t *testing.T) {
	if !assert {
		t.Fatal(msg)
	}
}
func TestPostgresSelect(t *testing.T) {
	// as per default policy actor table shuold not return
	// first_name.
	actor := struct {
		ActorID    int            `db:"actor_id"`
		FirstName  sql.NullString `db:"first_name"`
		LastName   sql.NullString `db:"last_name"`
		LastUpdate *time.Time     `db:"last_update"`
	}{}
	db := getDB("postgres", "admin", t)
	rows, err := db.Queryx("SELECT actor_id, first_name, last_name, last_update FROM actor limit 1")
	if err != nil {
		t.Fatal(err)
	}
	for rows.Next() {
		err := rows.StructScan(&actor)
		if err != nil {
			t.Fatal(err)
		}
		assert(actor.ActorID != 0, "expected actor id but got zero", t)
		assert(actor.FirstName.String == "", "expected first name to be empty", t)
		assert(actor.LastName.String != "", "expected last_name but got empty string", t)
		assert(actor.LastUpdate != nil, "expected last_update but got nil", t)
	}
}

func TestPostgresSelectWildCard(t *testing.T) {
	actor := struct {
		ActorID    int            `db:"actor_id"`
		FirstName  sql.NullString `db:"first_name"`
		LastName   sql.NullString `db:"last_name"`
		LastUpdate *time.Time     `db:"last_update"`
	}{}
	db := getDB("postgres", "admin", t)
	rows, err := db.Queryx("SELECT * FROM actor limit 1")
	if err != nil {
		t.Fatal(err)
	}
	for rows.Next() {
		err := rows.StructScan(&actor)
		if err != nil {
			t.Fatal(err)
		}
		assert(actor.ActorID != 0, "expected actor id but got zero", t)
		assert(actor.FirstName.String == "", "expected first name to be empty", t)
		assert(actor.LastName.String != "", "expected last_name but got empty string", t)
		assert(actor.LastUpdate != nil, "expected last_update but got nil", t)
	}
}

// divine-butterfly
// f3535f770dadb1

func TestInsertNotAllowed(t *testing.T) {
	db := getDB("postgres", "admin", t)
	_, err := db.Exec("insert into actor (first_name, last_name) values ('poonai', 'kuttypoonai');")
	assert(strings.Contains(err.Error(), "unauthorized insert"), "expected unathorized insert message", t)
}

func TestCopy(t *testing.T) {
	cmd := exec.Command("psql", `sslmode=disable host=localhost port=8081 dbname=postgres user=fragrant-sun password=d8c1e29d7e58be`, "-c", `\COPY actor(first_name,last_name) from 'data.csv' DELIMITER ',' CSV HEADER;`)
	output, err := cmd.CombinedOutput()
	assert(err != nil, "expected error but got nil", t)
	assert(strings.Contains(string(output), "unauthorized copy"), "unauthorized copy error message expected", t)
}

func TestCopySuccess(t *testing.T) {
	cred := Credentials["dev"]
	cmd := exec.Command("psql", fmt.Sprintf(`sslmode=disable host=localhost port=8081 dbname=postgres user=%s password=%s`, cred.UserName, cred.Password), "-c", `\COPY actor(first_name,last_name) from 'data.csv' DELIMITER ',' CSV HEADER;`)
	output, err := cmd.CombinedOutput()
	assert(err == nil, "expected nil but got error", t)
	assert(strings.Contains(string(output), "COPY 1"), "expected COPY 1", t)
}

func TestUpdate(t *testing.T) {
	db := getDB("postgres", "admin", t)
	_, err := db.Exec("update actor set first_name = 'poonai' where first_name = 'PENELOPE'")
	assert(strings.Contains(err.Error(), "unauthorized update"), "expected unathorized update message", t)
}

func errPrint(err error) string {
	if err == nil {
		return ""
	}
	return fmt.Sprintf("expecteed nil but got error: %s", err.Error())
}

func TestTransaction(t *testing.T) {
	// do a normal insert.
	db := getDB("postgres", "dev", t)
	tx := db.MustBegin()
	tx.MustExec("INSERT INTO actor (first_name, last_name) VALUES ('poonai2', 'poonai4');")
	err := tx.Commit()
	assert(err == nil, "expected nil but got error", t)

	// let's rollback if unauthorized insert comes.
	tx = db.MustBegin()
	tx.MustExec("INSERT INTO actor (first_name, last_name) VALUES ('poonai4', 'poonai5');")
	_, err = tx.Exec("insert into category (name) values ('sirkazhi')")
	assert(err != nil, "expected error but got nil", t)
	err = tx.Rollback()
	assert(err == nil, errPrint(err), t)

	// now we'll commmit and see whether values are retained.
	tx = db.MustBegin()
	tx.MustExec("INSERT INTO actor (first_name, last_name) VALUES ('poonai6', 'poonai7');")
	_, err = tx.Exec("insert into category (name) values ('sirkazhi')")
	assert(err != nil, "expected error but got nil", t)
	err = tx.Commit()
	assert(err == nil, errPrint(err), t)
	db = getDB("postgres", "admin", t)
	actor := struct {
		ActorID    int            `db:"actor_id"`
		FirstName  sql.NullString `db:"first_name"`
		LastName   sql.NullString `db:"last_name"`
		LastUpdate *time.Time     `db:"last_update"`
	}{}
	rows, err := db.Queryx("SELECT * FROM actor where first_name = 'poonai6' limit 1")
	if err != nil {
		t.Fatal(err)
	}
	for rows.Next() {
		err := rows.StructScan(&actor)
		if err != nil {
			t.Fatal(err)
		}
		assert(actor.LastName.String == "poonai7", "expected last_name to be poonai7", t)
	}
}

func TestControlplaneOffline(t *testing.T) {
	db := getDB("postgres", "dev", t)
	tx := db.MustBegin()
	cluster.controlplaneCmd.Process.Kill()
	time.Sleep(time.Second)
	tx.MustExec("INSERT INTO actor (first_name, last_name) VALUES ('poonai2', 'poonai4');")
	err := tx.Commit()
	assert(err == nil, "expected nil but got error", t)
	cmd := exec.Command("./controlplane", "--config-path", "./controlplane_config.yaml")
	err = cmd.Start()
	if err != nil {
		panic(err.Error())
	}
	time.Sleep(time.Second * 3)
	cluster.controlplaneCmd = cmd
}
