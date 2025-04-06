// use gametools::cards::*;
// use gametools::dice::*;
use gametools::*;

fn main() -> GameResult<()> {
   let mut deck = Deck::standard_52("main deck");
   let mut hand = CardHand::new("player_1");
   let mut discard_pile = Pile::new("discard");

   // shuffle and draw 7 cards into the hand
   deck.shuffle();
   hand.draw_cards_from(&mut deck, 7)?;
   println!("{hand}");

   // or deal from the deck to multiple hands
   let mut other_hands = vec![CardHand::new("player_2"), CardHand::new("player_3")];
   deck.deal_to_hands(&mut other_hands, 7)?;

   // count ranks and suits
   let num_spades = hand.count_suit(Suit::Spades);
   let num_queens = hand.count_rank(Rank::Queen);

   // search for a card in a hand
   let search_card = Card::new_temp(Rank::Ace, Suit::Clubs);
   let go_fish = hand.contains(&search_card);

   // move a card from hand to another hand or a pile; returns an error if card is not in hand
   hand.transfer_card(&search_card,&mut discard_pile)?;

   Ok(())
}