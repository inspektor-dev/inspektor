
import { createStore } from 'vuex'

import api from "@/api/api";

const store = createStore({
    state() {
        return {
            datasources: [],
            users: [],
        }
    },
    mutations: {
        setDatasource(state, datasource) {
            state.datasources = datasource
        },
        setUsers(state, users) {
            state.users = users
        }
    },
    actions: {
        async updateDatasource({ commit }) {
            let datasources = await api.getDatasources();
            commit("setDatasource", datasources)
        },
        async updateUsers({commit}) {
            let users = await api.getUsers()
            console.log(users)
            commit("setUsers", users)
        }
    }
})

export default store;