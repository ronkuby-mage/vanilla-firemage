                    <div class="overview" v-if="activeResultTab == 'overview'">
                        <div class="dps-overview" v-if="resultOpen">
                            <div class="players">
                                <div class="player" v-for="player in result.players">
                                    <div class="progress-wrapper">
                                        <progress-circle :value="playerDps(player) / result.dps" :animate="true" />
                                        <div class="center">
                                            <div class="value">
                                                <animate-number :end="playerDps(player) / result.dps * 100" :decimals="0" />%
                                            </div>
                                        </div>
                                    </div>
                                    <div class="info">
                                        <div class="name">{{ player.name }}</div>
                                        <div class="dps">
                                            <animate-number :end="playerDps(player)" />
                                        </div>
                                    </div>
                                </div>

                            </div>
                            <div class="total progress-wrapper">
                                <progress-circle :value="1" :animate="true" />
                                <div class="center">
                                    <div class="title">Total dps</div>
                                    <div class="value">
                                        <animate-number :end="result.dps" />
                                    </div>
                                    <div class="notice" v-if="result.iterations">{{ result.min_dps.toFixed() }} - {{ result.max_dps.toFixed() }}</div>
                                </div>
                            </div>
                            <div class="ignite progress-wrapper" v-if="result.ignite_dps">
                                <progress-circle :value="result.ignite_dps / result.dps" :animate="true" />
                                <div class="center">
                                    <div class="title">Ignite dps</div>
                                    <div class="value">
                                        <animate-number :end="result.ignite_dps" />
                                    </div>
                                </div>
                            </div>
                            <div class="info">
                                <table>
                                    <tbody>
                                        <tr><td>Execution time:</td><td>{{ result.time.toFixed(2) }}s</td></tr>
                                        <template v-if="result.iterations">
                                            <tr><td>Iterations:</td><td>{{ result.iterations }}</td></tr>
                                            <tr><td>Time / iteration:</td><td>{{ (result.time / result.iterations * 1000).toFixed(2) }}ms</td></tr>
                                        </template>
                                    </tbody>
                                </table>
                            </div>
                            <div class="histogram-section" v-if="result.iterations">
                                <histogram :data="histogramData" />
                            </div>
                        </div>
                    </div>