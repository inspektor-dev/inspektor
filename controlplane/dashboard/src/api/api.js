import axios from "axios";

let token = localStorage.getItem("access-token");
if (token != null) {
    axios.defaults.headers["Auth-Token"] = token;
}
//axios.defaults.baseURL = "http://localhost:3123"

axios.interceptors.response.use(function (res) {
    return res.data
}, function (error) {
    console.log("error pringing", error)
    Promise.reject(error)
})

export default {
    login: async (username, password) => {
        let res = await axios.post("/api/login", { username, password })
        return res.data.token
    },
    getDatasources: async () => {
        let res = await axios.get("/api/datasource")
        return res.data
    },
    addDatasource: async (data) => {
        await axios.post("/api/datasource", data)
    },
    addUser: async (data) => {
        await axios.post("/api/user", data)
    },
    getUsers: async () => {
        let res = await axios.get("/api/users")
        return res.data
    },
    getRoles: async () => {
        let res = await axios.get("/api/roles")
        return res.data
    },
    getSessions: async () => {
        let res = await axios.get("/api/session")
        return res.data
    },
    createSession: async (data) => {
        await axios.post("/api/session", data)
    },
    config: async () => {
        let res = await axios.get("/api/config")
        return res.data
    },
    refreshPolicy: async () => {
        await axios.post("/api/policy/nofification")
    },
    updateRoles: async (data) => {
        await axios.post("/api/roles", data)
    },
    createTempCredentials: async (data) => {
        await axios.post("/api/session/temp", data)
    },
    getTempCredentials: async (data) => {
        let res = await axios.get("/api/session/temp", data)
        return res.data
    },
    configureCloudWatch: async (data) => {
        await axios.post("/api/configure/cloudwatch", data)
    },
    configureAuditLog: async (data) => {
        await axios.post("/api/configure/auditlog", data)
    },
    createServiceAccount: async(data) => {
        await axios.post("/api/serviceaccount", data)
    },
    configureTeams: async (data) => {
        await axios.post("/api/configure/teams", data)
    },
    getServiceAccount: async () => {
        let res = await axios.get("api/serviceaccount")
        return res.data
    },
    deleteDatasource: async(data) => {
        await axios.delete("/api/datasource", {data})
    },
    intializeToken: async () => {
        let token = localStorage.getItem("access-token");
        if (token != null) {
            axios.defaults.headers["Auth-Token"] = token;
        }
    },
}