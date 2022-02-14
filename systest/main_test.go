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

func getDB(dbName string, t *testing.T) *sqlx.DB {
	psqlconn := fmt.Sprintf("host=%s port=%d user=%s password=%s dbname=%s sslmode=disable", "localhost", 8081, "fragrant-sun", "d8c1e29d7e58be", dbName)
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
	cluster := runCluster()
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
	db := getDB("postgres", t)
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

func TestInsertNotAllowed(t *testing.T) {
	db := getDB("postgres", t)
	_, err := db.Exec("insert into actor (first_name, last_name) values ('poonai', 'kuttypoonai');")
	assert(strings.Contains(err.Error(), "unauthorized insert"), "expected unathorized insert message", t)
}

func TestCopy(t *testing.T) {
	cmd := exec.Command("psql", `sslmode=disable host=localhost port=8081 dbname=postgres user=fragrant-sun password=d8c1e29d7e58be`, "-c", `\COPY actor(first_name,last_name) from 'data.csv' DELIMITER ',' CSV HEADER;`)
	output, err := cmd.CombinedOutput()
	assert(err != nil, "expected error but got nil", t)
	assert(strings.Contains(string(output), "unauthorized copy"), "unauthorized copy error message expected", t)
}

func TestUpdate(t *testing.T) {
	db := getDB("postgres", t)
	_, err := db.Exec("update actor set first_name = 'poonai' where first_name = 'PENELOPE'")
	assert(strings.Contains(err.Error(), "unauthorized update"), "expected unathorized update message", t)
}
