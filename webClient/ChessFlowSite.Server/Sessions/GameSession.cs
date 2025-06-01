using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.SignalR;

namespace ChessFlowSite.Server.Sessions
{
    public class GameSession
    {
        public int GameId { get; set; }
        public Player PlayerWhite { get; set; }
        public Player? PlayerBlack { get; set; } // can be null if it's a bot game

        public bool IsBot => PlayerBlack == null;
        public int? BotId { get; set; } // used for bot games

        public string? BotName => BotId switch
        {
            0 => "ChessFlow Engine",
            _ => "Unknown Bot"
        };
        public IHubCallerClients Clients { get; set; }

        public GameSession(int gameId, Player p1, Player p2, IHubCallerClients clients, int? botId = null)
        {
            GameId = gameId;
            Clients = clients;

            PlayerWhite = p1;
            PlayerBlack = p2;
            BotId = botId;
        }

        public string? GetOpponentId(string connectionId)
        {
            if (PlayerWhite?.ConnectionId == connectionId)
                return PlayerBlack?.ConnectionId;
            if (PlayerBlack?.ConnectionId == connectionId)
                return PlayerWhite?.ConnectionId;
            return null;
        }

        public bool ContainsPlayer(string connectionId) =>
            PlayerWhite?.ConnectionId == connectionId || PlayerBlack?.ConnectionId == connectionId;

        public Task SendMoveToGM(string move, string connectionId) {
            var clientColor = PlayerWhite.ConnectionId == connectionId ? "white" : "black";
            Console.WriteLine(String.Format("{0} from {1}", move, clientColor));
            return Clients.Client(clientColor).SendAsync("ReceiveMove", move);
        }
    }
}
