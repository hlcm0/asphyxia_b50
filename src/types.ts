export type PlayerSummary = {
  refid: string;
  name: string;
  sdvxId: number;
  scoreCount: number;
};

export type B50Card = {
  rank: number;
  mid: number;
  title: string;
  difficultyLabel: string;
  level: string;
  score: number;
  clearLamp: string;
  singleVf: string;
  jacketPath: string;
};

export type ScanResult = {
  version: number;
  players: PlayerSummary[];
};

export type AppSettings = {
  dataDir: string;
  savedataDir: string;
  backgroundImage: string;
};

export type B50Result = {
  version: number;
  player: PlayerSummary;
  totalVf: string;
  generatedAt: string;
  cards: B50Card[];
};
