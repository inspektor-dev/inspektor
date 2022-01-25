import axios from "axios";

let token = localStorage.getItem("access-token");
if (token != null) {
    axios.defaults.headers["Auth-Token"] = token;
}
axios.defaults.baseURL = "http://localhost:3123"
axios.interceptors.response.use(function (res) {
    return res.data
}, function (error) {
    console.log("error", error)
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
    intializeToken: () => {
        let token = localStorage.getItem("access-token");
        if (token != null) {
            axios.defaults.headers["Auth-Token"] = token;
        }
    }
}