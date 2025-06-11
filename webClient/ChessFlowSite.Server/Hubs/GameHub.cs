using ChessFlowSite.Server.Models;
using ChessFlowSite.Server.Services;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.SignalR;

namespace ChessFlowSite.Server.Hubs
{
    public class GameHub : Hub
    {
        private readonly GameManager _gameManager;
        private readonly UserManager<ApplicationUser> _userManager;
        private static readonly HashSet<string> JoinedConnections = new();

        public GameHub(GameManager gameManager, UserManager<ApplicationUser> userManager) {
            this._gameManager = gameManager;
            this._userManager = userManager;
        }

        public async Task JoinQueue(QueueJoinModel model) {
            var connectionId = Context.ConnectionId;

            Console.WriteLine(model.GuestUsername);
            Console.WriteLine("\n\n----------------\n\n");

            lock (JoinedConnections) {
                if (JoinedConnections.Contains(connectionId)) {
                    Clients.Caller.SendAsync("Error", "You are already in a queue or game.");
                    Console.WriteLine("You are already in a queue.");
                    return;
                }
                if (Context.User.Identity.IsAuthenticated || !(string.IsNullOrWhiteSpace(model.GuestUsername) || model.GuestUsername.Length > 31 || model.GuestElo == null)) {
                    JoinedConnections.Add(connectionId);
                }
            }

            Player player;
            if (Context.User.Identity.IsAuthenticated){
                var user = await _userManager.GetUserAsync(Context.User);
                player = new Player
                {
                    ConnectionId = connectionId,
                    Username = user.Name,
                    Elo = user.Elo,
                    isGuest = false,
                    Format = model.Format,
                    UserId = user.Id
                };
            }
            else {
                if (string.IsNullOrWhiteSpace(model.GuestUsername) || model.GuestUsername.Length > 31 || model.GuestElo == null) {
                    await Clients.Caller.SendAsync("Error", "Invalid guest username.");
                    return;
                }
                model.GuestUsername = model.GuestUsername.Trim();

                player = new Player {
                    ConnectionId = connectionId,
                    Username = model.GuestUsername,
                    Elo = (int)model.GuestElo,
                    isGuest = true,
                    Format = model.Format,
                    UserId = null
                };
            }

            if (model.IsBot)
            {
                await _gameManager.StartBotGame(player, model.BotId ?? 0, Clients);
            }
            else {
                await _gameManager.AddToQueueAsync(player, Clients);
            }
        }

        public async Task MakeMove(int gameId, string move)
        {
            await _gameManager.ProcessMoveAsync(gameId, Context.ConnectionId, move);
        }

        public async Task Resign() {
            await _gameManager.HandleDisconnectAsync(Context.ConnectionId);
        }

        public async Task OfferDraw() {
            await _gameManager.OfferDrawAsync(Context.ConnectionId);
        }

        public async Task RequestBoard() {
            await _gameManager.RequestBoardAsync(Context.ConnectionId);
        }

        public static Task RemoveConnection(string connectionId)
        {
            lock (JoinedConnections)
            {
                if (JoinedConnections.Contains(connectionId))
                {
                    JoinedConnections.Remove(connectionId);
                }
            }
            return Task.CompletedTask;
        }

        public override async Task OnDisconnectedAsync(Exception exception)
        {
            var connectionId = Context.ConnectionId;

            lock (JoinedConnections)
            {
                JoinedConnections.Remove(connectionId);
            }

            await _gameManager.HandleDisconnectAsync(connectionId);
            await base.OnDisconnectedAsync(exception);
        }
    }
    public class QueueJoinModel
    {
        public int? GuestElo { get; set; }
        public string? GuestUsername { get; set; }
        public string Format { get; set; }
        public bool IsBot { get; set; }
        public int? BotId { get; set; }
    }
}
