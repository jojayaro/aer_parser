(
  select * from 'data/csv/*WELLS*.csv'
  except
  select * from delta_scan('./data/delta/st1')
)
union all
(
  select * from delta_scan('./data/delta/st1')
  except
  select * from 'data/csv/*WELLS*.csv'
);