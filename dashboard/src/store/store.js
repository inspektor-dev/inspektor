
import { createStore } from 'vuex'

import api from "@/api/api";

const store = createStore({
    state() {
        return {
            datasources: [],
        }
    },
    mutations: {
        setDatasource(state, datasource) {
            state.datasources = datasource
        }
    },
    actions: {
        async updateDatasource({ commit }) {
            let datasources = await api.getDatasources();
            commit("setDatasource", datasources)
        }
    }
})

export default store;